#requires -Version 5.1
<#
.SYNOPSIS
Connect requested QHUD profiles through the provider's official login CLI.
.DESCRIPTION
Lists profiles by default. -Profile selects one registry key; -AllPending
connects each missing isolated login once, even when two Codex scopes share it.
No logout, custom OAuth request, persistent environment change, or default-login
replacement is performed. A browser login can choose the wrong account or
organization: only a matching local identity is registered for QHUD afterward.
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
    [Parameter(Mandatory = $true, ParameterSetName = 'Pending')]
    [switch]$AllPending,
    [Parameter(ParameterSetName = 'List')]
    [switch]$List,
    [string]$RegistryPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false

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
    param($Item)
    $Directory = Expand-ProfileDirectory $Item.directory
    $State = [pscustomobject]@{ State = 'login_required'; AccountId = $null; OrgId = $null }
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

function Invoke-ProfileLogin {
    param($Item)
    $Directory = Expand-ProfileDirectory $Item.directory
    if (Test-DefaultDirectory $Item.provider $Directory) {
        throw 'This helper will not replace a default login. Reconnect that profile in the provider CLI.'
    }
    Assert-OrdinaryProfileDirectory $Directory
    $Cli = Find-ProviderCli $Item.provider
    $null = [IO.Directory]::CreateDirectory($Directory)
    $Variable = if ($Item.provider -eq 'claude') { 'CLAUDE_CONFIG_DIR' } else { 'CODEX_HOME' }
    $Previous = [Environment]::GetEnvironmentVariable($Variable, 'Process')
    try {
        [Environment]::SetEnvironmentVariable($Variable, $Directory, 'Process')
        if ($Item.provider -eq 'claude') {
            & $Cli auth login --claudeai --email $Item.email
        } else {
            # A command-local file store keeps this login in its own CODEX_HOME.
            # It does not edit the provider's config.toml or system credential store.
            & $Cli -c 'cli_auth_credentials_store="file"' login
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

function Save-VerifiedProfiles {
    param([object[]]$Items)
    # Re-read after the browser flow so unrelated edits made while it was open
    # survive. Never serialize credential JSON or copy any token to the registry.
    $Registry = Read-Registry
    $Latest = @(Get-RegistryProfiles $Registry)
    foreach ($Original in $Items) {
        $Matches = @($Latest | Where-Object { $_.key -ceq $Original.key })
        if ($Matches.Count -ne 1) { throw 'The profile registry changed during login. Nothing was activated.' }
        $Item = $Matches[0]
        Assert-Profile $Item
        if ($Item.provider -cne $Original.provider -or $Item.email -cne $Original.email -or
            $Item.scope -cne $Original.scope -or -not (Test-SameDirectory $Item.directory $Original.directory)) {
            throw 'The profile registry changed during login. Nothing was activated.'
        }
        $State = Get-ProfileState $Item
        if ($State.State -ne 'ready') { throw 'The requested account and scope were not verified. Nothing was activated.' }
        if ($Item.provider -eq 'claude') {
            $Item | Add-Member -NotePropertyName expected_org_id -NotePropertyValue $State.OrgId -Force
        }
        if (-not (Test-DefaultDirectory $Item.provider $Item.directory)) {
            $Field = if ($Item.provider -eq 'claude') { 'claude_config_dirs' } else { 'codex_homes' }
            $Existing = @(Get-PropertyValue $Registry $Field) | Where-Object { $null -ne $_ }
            $AlreadyPresent = @($Existing | Where-Object { Test-SameDirectory $_ $Item.directory }).Count -gt 0
            if (-not $AlreadyPresent) {
                $Registry | Add-Member -NotePropertyName $Field -NotePropertyValue (@($Existing) + @($Item.directory)) -Force
            }
        }
    }
    $Temporary = $script:ResolvedRegistryPath + '.' + [Guid]::NewGuid().ToString('N') + '.tmp'
    try {
        [IO.File]::WriteAllText($Temporary, ($Registry | ConvertTo-Json -Depth 100), (New-Object Text.UTF8Encoding($false)))
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
    $Failures = 0
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
                Write-Host 'Choose that email and organization in the official provider login. Browser defaults may select another account.'
                Invoke-ProfileLogin $Item
            }
            # Exit code zero alone is not identity or scope verification.
            if (@($Group | Where-Object { (Get-ProfileState $_).State -ne 'ready' }).Count -gt 0) {
                throw 'Login did not match the requested email and organization. Nothing was activated.'
            }
            Save-VerifiedProfiles $Group
            Write-Host ('Verified: ' + (($Group | ForEach-Object { $_.label }) -join ', ') + '. Refresh that provider in QHUD.')
        } catch {
            $Failures++
            # Do not emit exception records: native/auth errors can contain private data.
            Write-Warning ('Profile not activated: ' + $Item.label + '. Check the requested email/scope and provider CLI, then retry. Default logins are left unchanged.')
        }
    }
    if ($Failures -gt 0) { exit 1 }
} catch {
    Write-Error 'Connection setup could not complete. Check the registry profiles and installed provider CLI. No credential details were printed.'
    exit 1
}
