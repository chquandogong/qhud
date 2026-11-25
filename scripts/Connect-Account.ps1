#requires -Version 5.1
<#
.SYNOPSIS
Connect requested QHUD profiles through the provider's official login CLI.
.DESCRIPTION
Lists profiles by default. -Profile selects one registry key; -AllPending
connects each missing isolated login once, even when two Codex scopes share it.
Use -Profile with -Reauthenticate to repair an expired matching default
identity. Interactive login runs in an empty private directory first. Only
credentials for the selected account and workspace are applied to its profile.
Run this interactively so the provider's own browser/login prompts are visible.
.EXAMPLE
.\scripts\Connect-Account.ps1 -List
.EXAMPLE
.\scripts\Connect-Account.ps1 -Profile codex-work-personal
.EXAMPLE
.\scripts\Connect-Account.ps1 -AllPending
#>
[CmdletBinding(DefaultParameterSetName = 'List')]
param(
    [Parameter(Mandatory = $true, ParameterSetName = 'Profile')]
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_-]{0,100}$')]
    [string]$Profile,
    [Parameter(ParameterSetName = 'Profile')]
    [switch]$Reauthenticate,
    [Parameter(Mandatory = $true, ParameterSetName = 'Pending')]
    [switch]$AllPending,
    [Parameter(ParameterSetName = 'List')]
    [switch]$List,
    [string]$RegistryPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false

# The widget can inherit a PowerShell 7 module path even though the native
# login console runs Windows PowerShell 5.1. Load its own ACL cmdlets.
if ($PSVersionTable.PSEdition -eq 'Desktop') {
    Import-Module (Join-Path $PSHOME 'Modules\Microsoft.PowerShell.Security\Microsoft.PowerShell.Security.psd1') -ErrorAction Stop
}

function Get-PropertyValue {
    param($Object, [string]$Name)
    if ($null -eq $Object) { return $null }
    $Property = $Object.PSObject.Properties[$Name]
    if ($null -ne $Property) { return $Property.Value }
    return $null
}

function Get-UserDirectory {
    if ($env:USERPROFILE) { return $env:USERPROFILE }
    if ($env:HOME) { return $env:HOME }
    throw 'The user home directory is unavailable.'
}

function Expand-ProfileDirectory {
    param([string]$Directory)
    if ([string]::IsNullOrWhiteSpace($Directory)) { throw 'A profile directory is missing.' }
    if ($Directory -eq '~') { $Directory = Get-UserDirectory }
    elseif ($Directory.StartsWith('~/')) {
        $Directory = Join-Path (Get-UserDirectory) $Directory.Substring(2)
    }
    # Relative directories change meaning with the shell's current directory.
    # Require explicit paths for a login destination.
    if (-not [IO.Path]::IsPathRooted($Directory)) {
        throw 'Profile directories must be absolute paths or start with ~/.'
    }
    return [IO.Path]::GetFullPath($Directory).TrimEnd('\', '/')
}

function Test-SameDirectory {
    param([string]$Left, [string]$Right)
    return [string]::Equals((Expand-ProfileDirectory $Left),
        (Expand-ProfileDirectory $Right), [StringComparison]::OrdinalIgnoreCase)
}

function Get-DefaultDirectory {
    param([string]$Provider)
    if ($Provider -eq 'claude') {
        if ($env:CLAUDE_CONFIG_DIR) { return Expand-ProfileDirectory $env:CLAUDE_CONFIG_DIR }
        return Join-Path (Get-UserDirectory) '.claude'
    }
    if ($env:CODEX_HOME) { return Expand-ProfileDirectory $env:CODEX_HOME }
    return Join-Path (Get-UserDirectory) '.codex'
}

function Test-DefaultDirectory {
    param([string]$Provider, [string]$Directory)
    # Protect both the effective profile and the ordinary desktop/CLI default.
    $Ordinary = Join-Path (Get-UserDirectory) ('.' + $Provider)
    return (Test-SameDirectory $Directory (Get-DefaultDirectory $Provider)) -or
        (Test-SameDirectory $Directory $Ordinary)
}

function Get-ClaudeConfigFile {
    param([string]$Directory)
    if (Test-SameDirectory $Directory (Get-DefaultDirectory 'claude')) {
        if ($env:CLAUDE_CONFIG_DIR) { return Join-Path $Directory '.claude.json' }
        return Join-Path (Get-UserDirectory) '.claude.json'
    }
    if (Test-SameDirectory $Directory (Join-Path (Get-UserDirectory) '.claude')) {
        return Join-Path (Get-UserDirectory) '.claude.json'
    }
    return Join-Path $Directory '.claude.json'
}

function Read-LocalJson {
    param([string]$Path)
    try {
        if (-not [IO.File]::Exists($Path)) { return $null }
        $Item = Get-Item -LiteralPath $Path -Force
        if ($Item.Length -gt 2097152) { return $null }
        return [IO.File]::ReadAllText($Path) | ConvertFrom-Json
    } catch { return $null }
}

function Read-Registry {
    $Value = Read-LocalJson $script:ResolvedRegistryPath
    if ($null -eq $Value -or $Value -isnot [pscustomobject]) {
        throw 'The QHUD account registry is missing or is not a readable JSON object.'
    }
    return $Value
}

function Get-RegistryProfiles {
    param($Registry)
    return @(Get-PropertyValue $Registry 'profiles') | Where-Object { $null -ne $_ }
}

function Assert-Profile {
    param($Item)
    foreach ($Name in @('key', 'provider', 'label', 'email', 'scope', 'directory')) {
        $Value = Get-PropertyValue $Item $Name
        if ($Value -isnot [string] -or [string]::IsNullOrWhiteSpace($Value)) {
            throw 'A requested profile has a missing or invalid field.'
        }
    }
    if ($Item.key -notmatch '^[A-Za-z0-9][A-Za-z0-9_-]{0,100}$' -or
        $Item.provider -notin @('claude', 'codex') -or
        $Item.scope -notin @('personal', 'team') -or
        $Item.email -notmatch '^[^\s@]+@[^\s@]+$') {
        throw 'A requested profile has an invalid key, provider, email, or scope.'
    }
    $Pin = Get-PropertyValue $Item 'expected_org_id'
    if ($null -ne $Pin -and ($Pin -isnot [string] -or [string]::IsNullOrWhiteSpace($Pin))) {
        throw 'A requested profile has an invalid organization pin.'
    }
    $null = Expand-ProfileDirectory $Item.directory
}

function Read-JwtClaims {
    param($Token)
    if ($Token -isnot [string] -or $Token.Length -gt 65536) { return $null }
    try {
        $Parts = $Token.Split('.')
        if ($Parts.Count -ne 3 -or -not $Parts[0] -or -not $Parts[1] -or -not $Parts[2]) {
            return $null
        }
        $Payload = $Parts[1].Replace('-', '+').Replace('_', '/')
        $Payload = $Payload.PadRight($Payload.Length + ((4 - $Payload.Length % 4) % 4), '=')
        return [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($Payload)) | ConvertFrom-Json
    } catch { return $null }
}

function Get-ClaudeScope {
    param($OrganizationType)
    switch ($OrganizationType) {
        { $_ -in @('claude_free', 'claude_pro', 'claude_max', 'claude_ultra') } { return 'personal' }
        { $_ -in @('claude_team', 'claude_enterprise') } { return 'team' }
        default { return $null }
    }
}

function Get-ProfileState {
    param($Item, [string]$Directory = (Expand-ProfileDirectory $Item.directory))
    $State = [pscustomobject]@{ State = 'login_required'; AccountId = $null; OrgId = $null; CodexScope = $null; ConflictingClaims = $false }
    $Email = $null
    $Expires = $null
    $HasCredential = $false
    if ($Item.provider -eq 'claude') {
        $Identity = Get-PropertyValue (Read-LocalJson (Get-ClaudeConfigFile $Directory)) 'oauthAccount'
        $Credential = Get-PropertyValue (Read-LocalJson (Join-Path $Directory '.credentials.json')) 'claudeAiOauth'
        $Email = Get-PropertyValue $Identity 'emailAddress'
        $State.AccountId = Get-PropertyValue $Identity 'accountUuid'
        $State.OrgId = Get-PropertyValue $Identity 'organizationUuid'
        $ActualScope = Get-ClaudeScope (Get-PropertyValue $Identity 'organizationType')
        $Token = Get-PropertyValue $Credential 'accessToken'
        $HasCredential = $Token -is [string] -and -not [string]::IsNullOrWhiteSpace($Token)
        $Expires = Get-PropertyValue $Credential 'expiresAt'
    } else {
        $Tokens = Get-PropertyValue (Read-LocalJson (Join-Path $Directory 'auth.json')) 'tokens'
        $State.AccountId = Get-PropertyValue $Tokens 'account_id'
        $Claims = Read-JwtClaims (Get-PropertyValue $Tokens 'id_token')
        $Workspace = Get-PropertyValue $Claims 'https://api.openai.com/auth'
        $ClaimId = Get-PropertyValue $Workspace 'chatgpt_account_id'
        if ($ClaimId -and $ClaimId -cne $State.AccountId) {
            $State.ConflictingClaims = $true
        } elseif ($ClaimId -and $ClaimId -ceq $State.AccountId) {
            $State.CodexScope = Get-CodexScope (Get-PropertyValue $Workspace 'chatgpt_plan_type')
        }
        $Email = Get-PropertyValue $Claims 'email'
        if ($Email -isnot [string] -or [string]::IsNullOrWhiteSpace($Email)) {
            $Email = Get-PropertyValue (Get-PropertyValue $Claims 'https://api.openai.com/profile') 'email'
        }
        $Token = Get-PropertyValue $Tokens 'access_token'
        $HasCredential = $Token -is [string] -and -not [string]::IsNullOrWhiteSpace($Token)
        $ExpirationSeconds = Get-PropertyValue (Read-JwtClaims $Token) 'exp'
        if ($null -ne $ExpirationSeconds -and $ExpirationSeconds -isnot [string]) {
            try { $Expires = [decimal]$ExpirationSeconds * 1000 } catch { $Expires = $null }
        }
    }
    if (-not $HasCredential -or $State.AccountId -isnot [string] -or
        [string]::IsNullOrWhiteSpace($State.AccountId) -or $Email -isnot [string] -or
        [string]::IsNullOrWhiteSpace($Email)) { return $State }
    if (-not [string]::Equals($Email.Trim(), $Item.email.Trim(), [StringComparison]::OrdinalIgnoreCase)) {
        $State.State = 'wrong_account'
        return $State
    }
    if ($Item.provider -eq 'claude') {
        $Pin = Get-PropertyValue $Item 'expected_org_id'
        if ($State.OrgId -isnot [string] -or [string]::IsNullOrWhiteSpace($State.OrgId) -or
            $ActualScope -cne $Item.scope -or ($Pin -and $Pin -cne $State.OrgId)) {
            $State.State = 'wrong_scope'
            return $State
        }
    }
    if ($null -ne $Expires -and $Expires -isnot [string] -and $Expires -gt 0 -and
        $Expires -le [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()) {
        $State.State = 'credential_expired'
        return $State
    }
    $State.State = 'ready'
    return $State
}

function Get-CodexScope {
    param($Plan)
    if ($Plan -in @('free', 'go', 'plus', 'prolite', 'pro', 'promax')) { return 'personal' }
    if ($Plan -in @('team', 'business', 'enterprise')) { return 'team' }
    return $null
}

function Test-LoginUuid {
    param($Value)
    return $Value -is [string] -and $Value -match '^[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}$'
}

function Test-ObservationTime {
    param($Value, [long]$Now)
    return ($Value -is [int] -or $Value -is [long]) -and $Value -gt 0 -and $Value -le $Now
}

function Get-LoginOrganization {
    param($Item, $Before)
    if ($Item.provider -eq 'claude') {
        $Pin = Get-PropertyValue $Item 'expected_org_id'
        if ($Pin) { return $Pin }
        if ($Before.State -in @('ready', 'credential_expired')) { return $Before.OrgId }
        return $null
    }
    $Now = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $Seat = Get-PropertyValue $Item 'seat_annotation'
    $Checked = Get-PropertyValue $Seat 'verified_at_ms'
    $Workspace = Get-PropertyValue $Seat 'workspace_id'
    if ($Item.scope -eq 'team' -and (Get-PropertyValue $Seat 'email') -eq $Item.email -and
        (Get-PropertyValue $Seat 'seat_type') -in @('standard', 'premium') -and
        (Test-ObservationTime $Checked $Now) -and (Test-LoginUuid $Workspace)) { return $Workspace }
    if ($Before.State -in @('ready', 'credential_expired') -and $Before.CodexScope -eq $Item.scope -and
        (Test-LoginUuid $Before.AccountId)) { return $Before.AccountId }
    $Store = Read-LocalJson (Join-Path (Split-Path -Parent $script:ResolvedRegistryPath) 'fetched-usage.json')
    $Snapshot = Get-PropertyValue $Store 'codex'
    $Checked = Get-PropertyValue $Snapshot 'fetched_at_ms'
    if (-not (Test-ObservationTime $Checked $Now)) { return $null }
    $Candidates = @(foreach ($Row in (Get-PropertyValue $Snapshot 'workspaces')) {
        $Id = Get-PropertyValue $Row 'account_id'
        if ((Get-PropertyValue $Row 'email') -eq $Item.email -and
            (Get-CodexScope (Get-PropertyValue $Row 'plan_type')) -eq $Item.scope -and (Test-LoginUuid $Id)) { $Id }
    })
    $Unique = @($Candidates | Sort-Object -Unique)
    if ($Unique.Count -eq 1) { return $Unique[0] }
    if ($Unique.Count -gt 1) {
        # Email and plan alone cannot select among these existing workspaces.
        throw [System.Security.Authentication.AuthenticationException]::new('wrong_scope')
    }
    return $null
}

function Find-ProviderCli {
    param([string]$Provider)
    if ($Provider -eq 'codex' -and $env:QHUD_CODEX_BIN -and [IO.File]::Exists($env:QHUD_CODEX_BIN)) {
        return $env:QHUD_CODEX_BIN
    }
    $Command = Get-Command $Provider -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($Command) { return $Command.Source }
    if ($Provider -eq 'claude') {
        $Candidate = Join-Path (Get-UserDirectory) '.local\bin\claude.exe'
        if ([IO.File]::Exists($Candidate)) { return $Candidate }
    } elseif ($env:LOCALAPPDATA) {
        foreach ($Relative in @('Programs\OpenAI\Codex\bin', 'OpenAI\Codex\bin')) {
            $Root = Join-Path $env:LOCALAPPDATA $Relative
            $Candidate = Join-Path $Root 'codex.exe'
            if ([IO.File]::Exists($Candidate)) { return $Candidate }
            if ([IO.Directory]::Exists($Root)) {
                $Candidates = @(Get-ChildItem -LiteralPath $Root -Directory -ErrorAction SilentlyContinue |
                    ForEach-Object { Join-Path $_.FullName 'codex.exe' } |
                    Where-Object { [IO.File]::Exists($_) } |
                    Sort-Object { [IO.File]::GetLastWriteTimeUtc($_) } -Descending)
                if ($Candidates.Count -gt 0) { return $Candidates[0] }
            }
        }
    }
    throw 'The official provider CLI was not found. Install it and run this helper again.'
}

function Assert-OrdinaryProfileDirectory {
    param([string]$Directory)
    # A junction into a default profile is still the default login. Refuse
    # redirected ancestors rather than risking a hidden destination change.
    $Current = $Directory
    while ($Current) {
        if (Test-Path -LiteralPath $Current) {
            $Entry = Get-Item -LiteralPath $Current -Force
            if (-not $Entry.PSIsContainer -or
                ($Entry.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
                throw 'Login profile directories must not be files, links, or junctions.'
            }
        }
        $Parent = Split-Path -Parent $Current
        if ($Parent -eq $Current) { break }
        $Current = $Parent
    }
}

function Invoke-ProviderLogin {
    param($Item, [string]$Directory, $Organization)
    $Cli = Find-ProviderCli $Item.provider
    $Variable = if ($Item.provider -eq 'claude') { 'CLAUDE_CONFIG_DIR' } else { 'CODEX_HOME' }
    $Previous = [Environment]::GetEnvironmentVariable($Variable, 'Process')
    try {
        [Environment]::SetEnvironmentVariable($Variable, $Directory, 'Process')
        if ($Item.provider -eq 'claude') {
            if (Test-LoginUuid $Organization) {
                $Settings = @{ forceLoginOrgUUID = $Organization } | ConvertTo-Json -Compress
                [IO.File]::WriteAllText((Join-Path $Directory 'settings.json'), $Settings)
            }
            & $Cli auth login --claudeai --email $Item.email
        } else {
            $Options = @('-c', 'cli_auth_credentials_store="file"')
            if (Test-LoginUuid $Organization) { $Options += @('-c', ('forced_chatgpt_workspace_id="' + $Organization + '"')) }
            & $Cli @Options login
        }
        if ($LASTEXITCODE -ne 0) { throw 'The provider login did not complete successfully.' }
    } finally {
        # PowerShell can coerce $null to an empty string for this .NET overload.
        # Modern .NET preserves an empty environment value, so explicitly pass
        # a null string when the variable was originally absent.
        if ($null -eq $Previous) {
            [Environment]::SetEnvironmentVariable($Variable, [NullString]::Value, 'Process')
        } else {
            [Environment]::SetEnvironmentVariable($Variable, $Previous, 'Process')
        }
    }
}

function Assert-OrdinaryLoginFile {
    param([string]$Path)
    Assert-OrdinaryProfileDirectory (Split-Path -Parent $Path)
    $Entry = Get-Item -LiteralPath $Path -Force -ErrorAction SilentlyContinue
    if ($Entry -and ($Entry.PSIsContainer -or ($Entry.Attributes -band [IO.FileAttributes]::ReparsePoint))) {
        throw 'Login files must be regular files.'
    }
}

function Read-LoginFile {
    param([string]$Path)
    Assert-OrdinaryLoginFile $Path
    if (-not [IO.File]::Exists($Path)) { return $null }
    $Text = [IO.File]::ReadAllText($Path)
    if ($Text.Length -gt 2097152 -or ($Text | ConvertFrom-Json) -isnot [pscustomobject]) {
        throw 'A provider file could not be safely updated.'
    }
    return $Text
}

function Merge-LoginField {
    param([string]$Original, $Source, [string]$Field)
    $Data = if ($Original) { $Original | ConvertFrom-Json } else { [pscustomobject]@{} }
    $NewValue = Get-PropertyValue $Source $Field
    if ($NewValue -isnot [pscustomobject]) { throw 'The provider login is incomplete.' }
    $Data | Add-Member -NotePropertyName $Field -NotePropertyValue $NewValue -Force
    return ($Data | ConvertTo-Json -Depth 100)
}

function Set-LoginFile {
    param([string]$Path, $Expected, $NewText)
    if (-not [string]::Equals((Read-LoginFile $Path), $Expected, [StringComparison]::Ordinal) -or
        [IO.File]::Exists($Path) -ne ($null -ne $Expected)) { throw 'The provider login changed during sign-in.' }
    if ($null -eq $NewText) { [IO.File]::Delete($Path); return }
    $Temporary = $Path + '.' + [guid]::NewGuid().ToString('N') + '.tmp'
    try {
        [IO.File]::WriteAllText($Temporary, $NewText, (New-Object Text.UTF8Encoding($false)))
        if ($null -ne $Expected) { [IO.File]::Replace($Temporary, $Path, [NullString]::Value) }
        else { [IO.File]::Move($Temporary, $Path) }
    } finally {
        if ([IO.File]::Exists($Temporary)) { [IO.File]::Delete($Temporary) }
    }
}

function Clear-LoginStage {
    param([string]$Path)
    # Never traverse junctions or links created by a provider in its home.
    foreach ($Child in (Get-ChildItem -LiteralPath $Path -Force)) {
        if ($Child.PSIsContainer) {
            if ($Child.Attributes -band [IO.FileAttributes]::ReparsePoint) { [IO.Directory]::Delete($Child.FullName) }
            else { Clear-LoginStage $Child.FullName }
        } else { [IO.File]::Delete($Child.FullName) }
    }
    [IO.Directory]::Delete($Path)
}

function Invoke-ProfileLogin {
    param($Item, [object[]]$Group = @($Item), [switch]$Reauthenticate)
    $Directory = Expand-ProfileDirectory $Item.directory
    $Before = Get-ProfileState $Item
    if ((Test-DefaultDirectory $Item.provider $Directory) -and
        (-not $Reauthenticate -or $Before.State -ne 'credential_expired')) {
        throw 'An expired matching default login is required for a targeted repair.'
    }
    Assert-OrdinaryProfileDirectory $Directory
    $Organization = Get-LoginOrganization $Item $Before
    $Name = if ($Item.provider -eq 'claude') { '.credentials.json' } else { 'auth.json' }
    $CredentialPath = Join-Path $Directory $Name
    $Original = Read-LoginFile $CredentialPath
    $ConfigPath = if ($Item.provider -eq 'claude') { Get-ClaudeConfigFile $Directory }
    $OriginalConfig = if ($ConfigPath) { Read-LoginFile $ConfigPath }
    # Start empty: some CLIs revoke existing credentials at login startup.
    $TempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\', '/')
    Assert-OrdinaryProfileDirectory $TempRoot
    $Stage = Join-Path $TempRoot ('qhud-login-' + [Guid]::NewGuid().ToString('N'))
    $null = New-Item -ItemType Directory -Path $Stage
    try {
        $Access = Get-Acl -LiteralPath $Stage
        $Access.SetAccessRuleProtection($true, $false)
        $User = [Security.Principal.WindowsIdentity]::GetCurrent().User
        $Access.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($User, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow'))
        Set-Acl -LiteralPath $Stage -AclObject $Access
        Invoke-ProviderLogin $Item $Stage $Organization
        Assert-OrdinaryLoginFile (Join-Path $Stage $Name)
        if ($ConfigPath) { Assert-OrdinaryLoginFile (Join-Path $Stage '.claude.json') }
        $Verified = Get-ProfileState $Item $Stage
        if ($Verified.State -eq 'wrong_account' -or ($Item.provider -eq 'claude' -and $Before.State -eq 'credential_expired' -and
            $Before.AccountId -cne $Verified.AccountId -and $Verified.AccountId)) {
            throw [System.Security.Authentication.AuthenticationException]::new('wrong_account')
        }
        if ($Verified.State -eq 'wrong_scope' -or ($ConfigPath -and $Organization -and $Verified.OrgId -cne $Organization)) {
            throw [System.Security.Authentication.AuthenticationException]::new('wrong_scope')
        }
        if ($Verified.State -ne 'ready') { throw 'The provider did not return a usable login.' }
        if ($Item.provider -eq 'codex' -and ($Verified.ConflictingClaims -or ($Organization -and $Verified.AccountId -ne $Organization) -or
            ($Verified.CodexScope -and $Verified.CodexScope -ne $Item.scope) -or (-not $Organization -and -not $Verified.CodexScope))) {
            throw [System.Security.Authentication.AuthenticationException]::new('wrong_scope')
        }
        $StagedCredential = Read-LoginFile (Join-Path $Stage $Name)
        $NewConfig = $null
        if ($ConfigPath) {
            $StagedConfig = (Read-LoginFile (Join-Path $Stage '.claude.json')) | ConvertFrom-Json
            # For a known identity the credential alone needs replacement. The
            # regular .claude.json also holds unrelated, frequently updated state.
            if ($Before.State -notin @('ready', 'credential_expired')) {
                $NewConfig = Merge-LoginField $OriginalConfig $StagedConfig 'oauthAccount'
            }
            $StagedCredential = Merge-LoginField $Original ($StagedCredential | ConvertFrom-Json) 'claudeAiOauth'
        }
        $Pending = [pscustomobject]@{ CredentialPath=$CredentialPath; Original=$Original; Credential=$StagedCredential;
            ConfigPath=$ConfigPath; OriginalConfig=$OriginalConfig; Config=$NewConfig; Before=$Before; Target=$Item; Organization=$Organization }
        Save-VerifiedProfiles $Group -PendingLogin $Pending
    } finally {
        $ResolvedStage = [IO.Path]::GetFullPath($Stage)
        if ((Split-Path -Parent $ResolvedStage) -ceq $TempRoot -and (Split-Path -Leaf $ResolvedStage) -match '^qhud-login-[a-f0-9]{32}$') {
            Assert-OrdinaryProfileDirectory $ResolvedStage
            Clear-LoginStage $ResolvedStage
        }
    }
}

function Save-VerifiedProfiles {
    param([object[]]$Items, $PendingLogin)
    # Serialize short activation writes by concurrent helpers after browser
    # login, without holding a lock while the operator is signing in.
    $Sha = [Security.Cryptography.SHA256]::Create()
    try {
        $PathHash = [BitConverter]::ToString($Sha.ComputeHash([Text.Encoding]::UTF8.GetBytes(
            $script:ResolvedRegistryPath.ToUpperInvariant()))).Replace('-', '')
    } finally { $Sha.Dispose() }
    $Mutex = New-Object Threading.Mutex($false, ('Local\qhud-connect-' + $PathHash))
    $Locked = $false
    try {
        try { $Locked = $Mutex.WaitOne(10000) } catch [Threading.AbandonedMutexException] { $Locked = $true }
        if (-not $Locked) { throw 'The account registry is busy. Retry the connection.' }
        if ($null -eq $PendingLogin) { Save-VerifiedRegistry $Items; return }
        $P = $PendingLogin
        $Latest = @(Get-RegistryProfiles (Read-Registry))
        $Shared = @($Latest | Where-Object { $_.provider -ceq $P.Target.provider -and (Test-SameDirectory $_.directory $P.Target.directory) })
        if ($Shared.Count -ne $Items.Count) { throw 'The shared profile group changed during login.' }
        foreach ($Original in $Items) {
            $Matching = @(Get-MatchingRegistryProfile $Shared $Original)
            if ($Matching.Count -ne 1 -or ($Original.provider -eq 'claude' -and
                (Get-PropertyValue $Matching[0] 'expected_org_id') -cne (Get-PropertyValue $Original 'expected_org_id'))) {
                throw 'The profile changed during sign-in.'
            }
        }
        $Current = Get-ProfileState $P.Target
        $CurrentTarget = Get-MatchingRegistryProfile $Shared $P.Target
        if ($Current.State -cne $P.Before.State -or $Current.AccountId -cne $P.Before.AccountId -or $Current.OrgId -cne $P.Before.OrgId -or
            (Get-LoginOrganization $CurrentTarget $Current) -cne $P.Organization) { throw 'The login target changed during sign-in.' }
        $null = [IO.Directory]::CreateDirectory((Split-Path -Parent $P.CredentialPath))
        $ConfigWritten = $false
        $CredentialWritten = $false
        $OldOauthHidden = $false
        $PriorCredential = $P.Original
        try {
            if ($null -ne $P.Config) {
                # Readers can observe the two files independently. Keep the
                # old member's token unavailable whenever the identity changes.
                if ($null -ne $P.Original) {
                    $Disabled = $P.Original | ConvertFrom-Json
                    $Disabled.PSObject.Properties.Remove('claudeAiOauth')
                    $PriorCredential = $Disabled | ConvertTo-Json -Depth 100
                    Set-LoginFile $P.CredentialPath $P.Original $PriorCredential
                    $OldOauthHidden = $true
                }
                Set-LoginFile $P.ConfigPath $P.OriginalConfig $P.Config
                $ConfigWritten = $true
            }
            Set-LoginFile $P.CredentialPath $PriorCredential $P.Credential
            $CredentialWritten = $true
            Save-VerifiedRegistry $Items
        } catch {
            # Each reversal requires the preceding step to succeed. If another
            # writer has changed a file, never pair its identity with our token.
            if ($CredentialWritten) { Set-LoginFile $P.CredentialPath $P.Credential $PriorCredential }
            if ($ConfigWritten) { Set-LoginFile $P.ConfigPath $P.Config $P.OriginalConfig }
            if ($OldOauthHidden) {
                if (-not [string]::Equals((Read-LoginFile $P.ConfigPath), $P.OriginalConfig, [StringComparison]::Ordinal)) {
                    throw 'The identity changed while rolling back the login.'
                }
                Set-LoginFile $P.CredentialPath $PriorCredential $P.Original
            }
            throw
        }
    } finally {
        if ($Locked) { $Mutex.ReleaseMutex() }
        $Mutex.Dispose()
    }
}

function Get-MatchingRegistryProfile {
    param([object[]]$Latest, $Original)
    $Matches = @($Latest | Where-Object { $_.key -ceq $Original.key })
    if ($Matches.Count -ne 1) { throw 'The profile registry changed during login.' }
    $Item = $Matches[0]
    Assert-Profile $Item
    if ($Item.provider -cne $Original.provider -or $Item.email -cne $Original.email -or
        $Item.scope -cne $Original.scope -or
        ((Get-PropertyValue $Original 'expected_org_id') -and
         (Get-PropertyValue $Item 'expected_org_id') -cne $Original.expected_org_id) -or
        -not (Test-SameDirectory $Item.directory $Original.directory)) {
        throw 'The profile registry changed during login.'
    }
    return $Item
}

function Save-VerifiedRegistry {
    param([object[]]$Items)
    $OriginalJson = [IO.File]::ReadAllText($script:ResolvedRegistryPath)
    $Registry = $OriginalJson | ConvertFrom-Json
    $Changed = $false
    $Latest = @(Get-RegistryProfiles $Registry)
    foreach ($Original in $Items) {
        $Item = Get-MatchingRegistryProfile $Latest $Original
        $State = Get-ProfileState $Item
        if ($State.State -ne 'ready') { throw 'The requested account and scope were not verified. Nothing was activated.' }
        if ($Item.provider -eq 'claude') {
            if ((Get-PropertyValue $Item 'expected_org_id') -cne $State.OrgId) {
                $Item | Add-Member -NotePropertyName expected_org_id -NotePropertyValue $State.OrgId -Force
                $Changed = $true
            }
        }
        if (-not (Test-DefaultDirectory $Item.provider $Item.directory)) {
            $Field = if ($Item.provider -eq 'claude') { 'claude_config_dirs' } else { 'codex_homes' }
            $Existing = @(Get-PropertyValue $Registry $Field) | Where-Object { $null -ne $_ }
            $AlreadyPresent = @($Existing | Where-Object { Test-SameDirectory $_ $Item.directory }).Count -gt 0
            if (-not $AlreadyPresent) {
                $Registry | Add-Member -NotePropertyName $Field -NotePropertyValue (@($Existing) + @($Item.directory)) -Force
                $Changed = $true
            }
        }
    }
    if (-not $Changed) { return }
    $Temporary = $script:ResolvedRegistryPath + '.' + [Guid]::NewGuid().ToString('N') + '.tmp'
    try {
        [IO.File]::WriteAllText($Temporary, ($Registry | ConvertTo-Json -Depth 100), (New-Object Text.UTF8Encoding($false)))
        if (-not [string]::Equals($OriginalJson, [IO.File]::ReadAllText($script:ResolvedRegistryPath), [StringComparison]::Ordinal)) {
            throw 'The account registry changed during activation. Retry the connection.'
        }
        [IO.File]::Replace($Temporary, $script:ResolvedRegistryPath, [NullString]::Value)
    } finally {
        if ([IO.File]::Exists($Temporary)) { Remove-Item -LiteralPath $Temporary -Force }
    }
}

try {
    if (-not $RegistryPath) {
        if ($env:QHUD_CONFIG_DIR) { $ConfigDirectory = $env:QHUD_CONFIG_DIR }
        elseif ($env:XDG_CONFIG_HOME) { $ConfigDirectory = Join-Path $env:XDG_CONFIG_HOME 'qhud' }
        else {
            $Legacy = Join-Path (Get-UserDirectory) '.config\qhud'
            if ([IO.Directory]::Exists($Legacy)) { $ConfigDirectory = $Legacy }
            elseif ($env:APPDATA) { $ConfigDirectory = Join-Path $env:APPDATA 'qhud' }
            else { $ConfigDirectory = $Legacy }
        }
        $RegistryPath = Join-Path $ConfigDirectory 'accounts.json'
    }
    $script:ResolvedRegistryPath = [IO.Path]::GetFullPath($RegistryPath)
    $Profiles = @(Get-RegistryProfiles (Read-Registry))
    foreach ($Item in $Profiles) { Assert-Profile $Item }
    if (@($Profiles | Group-Object key | Where-Object { $_.Count -gt 1 }).Count -gt 0) {
        throw 'Profile keys must be unique in the account registry.'
    }
    if ($PSCmdlet.ParameterSetName -eq 'List') {
        $Profiles | ForEach-Object {
            [pscustomobject]@{ Key = $_.key; Provider = $_.provider; Label = $_.label;
                Email = $_.email; Scope = $_.scope; State = (Get-ProfileState $_).State }
        } | Format-Table -AutoSize
        return
    }
    $Selected = if ($Profile) { @($Profiles | Where-Object { $_.key -ceq $Profile }) } else { $Profiles }
    if (@($Selected).Count -eq 0) { throw 'No matching connection profile was found.' }
    $Handled = New-Object 'Collections.Generic.HashSet[string]' ([StringComparer]::OrdinalIgnoreCase)
    $FailureCode = 0
    foreach ($Item in $Selected) {
        $Directory = Expand-ProfileDirectory $Item.directory
        $GroupKey = $Item.provider + ':' + $Directory
        if (-not $Handled.Add($GroupKey)) { continue }
        # Include both Codex scopes sharing one login, even for -Profile.
        $Group = @($Profiles | Where-Object {
            $_.provider -ceq $Item.provider -and (Test-SameDirectory $_.directory $Directory)
        })
        try {
            foreach ($Member in $Group) {
                if (-not [string]::Equals($Member.email, $Item.email, [StringComparison]::OrdinalIgnoreCase) -or
                    ($Item.provider -eq 'claude' -and $Member.scope -cne $Item.scope)) {
                    throw 'Profiles sharing one directory must request the same login and Claude organization scope.'
                }
            }
            $Pending = @($Group | Where-Object { (Get-ProfileState $_).State -ne 'ready' })
            if ($Pending.Count -gt 0) {
                Write-Host ('Connecting ' + $Item.label + ' (' + $Item.provider + ', ' + $Item.email + ', ' + $Item.scope + ').')
                Write-Host ('Sign in as ' + $Item.email + ' in the official login window. Only the requested account and workspace can be applied.')
                Invoke-ProfileLogin $Item -Group $Group -Reauthenticate:$Reauthenticate
            } else {
                Save-VerifiedProfiles $Group
            }
            Write-Host ('Verified: ' + (($Group | ForEach-Object { $_.label }) -join ', ') + '. Refresh that provider in QHUD.')
        } catch {
            $FailureCode = 1
            if ($_.Exception -is [System.Security.Authentication.AuthenticationException]) {
                $FailureCode = 2
                if ($Reauthenticate) { $FailureCode = if ($_.Exception.Message -ceq 'wrong_account') { 3 } else { 4 } }
            }
            # Do not emit exception records: native/auth errors can contain private data.
            Write-Warning ('Profile not activated: ' + $Item.label + '. Check the requested email, organization and provider CLI, then retry.')
        }
    }
    if ($FailureCode -gt 0) { exit $FailureCode }
} catch {
    Write-Error 'Connection setup could not complete. Check the registry profiles and installed provider CLI. No credential details were printed.'
    exit 1
}
