#requires -Version 5.1
# Synthetic local fixtures only. No provider CLI, browser, or network is used.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$Helper = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\scripts\Connect-Account.ps1'))
$ParseTokens = $null
$ParseErrors = $null
$Ast = [System.Management.Automation.Language.Parser]::ParseFile($Helper, [ref]$ParseTokens, [ref]$ParseErrors)
if ($ParseErrors.Count) { throw 'Connection helper does not parse.' }
# Import function definitions without invoking the script's login entry point.
$Functions = $Ast.FindAll({ param($Node)
    $Node -is [System.Management.Automation.Language.FunctionDefinitionAst]
}, $false)
foreach ($Function in $Functions) { . ([scriptblock]::Create($Function.Extent.Text)) }

$TestRoot = Join-Path ([IO.Path]::GetTempPath()) ('qhud-connection-test-' + [Guid]::NewGuid().ToString('N'))
$null = [IO.Directory]::CreateDirectory($TestRoot)
$Utf8 = New-Object Text.UTF8Encoding($false)
function Write-TestJson($Path, $Object) {
    [IO.File]::WriteAllText($Path, ($Object | ConvertTo-Json -Depth 20), $Utf8)
}
function Assert-Test($Condition, $Message) {
    if (-not $Condition) { throw $Message }
}
function Test-Token($Claims) {
    $Json = $Claims | ConvertTo-Json -Compress
    $Payload = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($Json)).TrimEnd('=').Replace('+', '-').Replace('/', '_')
    return 'synthetic.' + $Payload + '.synthetic'
}
function Restore-TestEnvironment([string]$Name, $Value) {
    if ($null -eq $Value) {
        [Environment]::SetEnvironmentVariable($Name, [NullString]::Value, 'Process')
    } else {
        [Environment]::SetEnvironmentVariable($Name, $Value, 'Process')
    }
}

try {
    $ClaudeDir = Join-Path $TestRoot 'claude-team'
    $null = [IO.Directory]::CreateDirectory($ClaudeDir)
    $Requested = [pscustomobject]@{ key = 'test-team'; provider = 'claude'; label = 'Test team';
        email = 'person@example.test'; scope = 'team'; directory = $ClaudeDir }
    $Config = @{ oauthAccount = @{ emailAddress = 'person@example.test'; accountUuid = 'synthetic-account';
        organizationUuid = 'synthetic-org'; organizationType = 'claude_team' } }
    Write-TestJson (Join-Path $ClaudeDir '.claude.json') $Config
    $Credential = @{ claudeAiOauth = @{ accessToken = 'SYNTHETIC-NONCREDENTIAL'; expiresAt = 4102444800000 } }
    Write-TestJson (Join-Path $ClaudeDir '.credentials.json') $Credential
    Assert-Test ((Get-ProfileState $Requested).State -eq 'ready') 'Matching Claude team was rejected.'
    $Requested.scope = 'personal'
    Assert-Test ((Get-ProfileState $Requested).State -eq 'wrong_scope') 'Wrong Claude scope was accepted.'
    $Requested.scope = 'team'
    $Requested.email = 'other@example.test'
    Assert-Test ((Get-ProfileState $Requested).State -eq 'wrong_account') 'Wrong Claude email was accepted.'
    $Requested.email = 'person@example.test'
    $Requested | Add-Member -NotePropertyName expected_org_id -NotePropertyValue 'different-org'
    Assert-Test ((Get-ProfileState $Requested).State -eq 'wrong_scope') 'Wrong Claude organization pin was accepted.'
    $Requested.PSObject.Properties.Remove('expected_org_id')
    $Credential.claudeAiOauth.expiresAt = 1
    Write-TestJson (Join-Path $ClaudeDir '.credentials.json') $Credential
    Assert-Test ((Get-ProfileState $Requested).State -eq 'credential_expired') 'Expired Claude credential was accepted.'
    $Credential.claudeAiOauth.expiresAt = 4102444800000
    Write-TestJson (Join-Path $ClaudeDir '.credentials.json') $Credential

    $script:ResolvedRegistryPath = Join-Path $TestRoot 'accounts.json'
    Write-TestJson $script:ResolvedRegistryPath @{ profiles = @($Requested); custom = @{ preserve = 'yes' }; labels = @{ existing = 'keep' } }
    Save-VerifiedProfiles @($Requested)
    $Saved = Read-Registry
    Assert-Test ($Saved.custom.preserve -eq 'yes') 'Unmodelled registry key was dropped.'
    Assert-Test ($Saved.labels.existing -eq 'keep') 'Existing registry label was changed.'
    Assert-Test ($Saved.profiles[0].expected_org_id -eq 'synthetic-org') 'Verified organization was not pinned.'
    Assert-Test (@($Saved.claude_config_dirs).Count -eq 1) 'Verified directory was not activated.'
    Save-VerifiedProfiles @($Requested)
    Assert-Test (@((Read-Registry).claude_config_dirs).Count -eq 1) 'Repeated activation duplicated a directory.'
    $Before = [IO.File]::ReadAllText($script:ResolvedRegistryPath)
    $Config.oauthAccount.emailAddress = 'wrong@example.test'
    Write-TestJson (Join-Path $ClaudeDir '.claude.json') $Config
    $Rejected = $false
    try { Save-VerifiedProfiles @($Requested) } catch { $Rejected = $true }
    Assert-Test $Rejected 'Wrong login was activated.'
    Assert-Test ($Before -ceq [IO.File]::ReadAllText($script:ResolvedRegistryPath)) 'Failed verification changed the registry.'

    # The real entry point skips login when a shared Codex credential matches.
    $CodexDir = Join-Path $TestRoot 'codex-shared'
    $null = [IO.Directory]::CreateDirectory($CodexDir)
    Write-TestJson (Join-Path $CodexDir 'auth.json') @{ tokens = @{ account_id = 'synthetic-personal';
        id_token = (Test-Token @{ email = 'person@example.test' });
        access_token = (Test-Token @{ exp = 4102444800 }) } }
    $Shared = @('personal', 'team') | ForEach-Object {
        [pscustomobject]@{ key = ('test-' + $_); provider = 'codex'; label = ('Test ' + $_);
            email = 'person@example.test'; scope = $_; directory = $CodexDir }
    }
    foreach ($Item in $Shared) {
        Assert-Test ((Get-ProfileState $Item).State -eq 'ready') 'Synthetic Codex precondition failed.'
    }
    Write-TestJson $script:ResolvedRegistryPath @{ profiles = @($Shared); keep = 'unrelated' }
    # Even a regression in the entry point must never launch a real provider.
    $FakeCli = Join-Path $TestRoot 'never-login.cmd'
    [IO.File]::WriteAllText($FakeCli, '@exit /b 97', $Utf8)
    $PreviousCli = [Environment]::GetEnvironmentVariable('QHUD_CODEX_BIN', 'Process')
    try {
        [Environment]::SetEnvironmentVariable('QHUD_CODEX_BIN', $FakeCli, 'Process')
        $CurrentPowerShell = [Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
        & $CurrentPowerShell -NoLogo -NoProfile -ExecutionPolicy Bypass -File $Helper -AllPending -RegistryPath $script:ResolvedRegistryPath | Out-Null
        Assert-Test ($LASTEXITCODE -eq 0) 'Shared ready Codex activation failed.'
    } finally {
        Restore-TestEnvironment 'QHUD_CODEX_BIN' $PreviousCli
    }
    $Saved = Read-Registry
    Assert-Test (@($Saved.codex_homes).Count -eq 1) 'Shared Codex scopes registered multiple homes.'
    Assert-Test ($Saved.profiles.Count -eq 2) 'Shared Codex activation removed a scope.'
    Assert-Test ($Saved.keep -eq 'unrelated') 'Shared Codex activation dropped an unrelated key.'

    # Replace the CLI with a local function to exercise failure and restoration.
    function Find-ProviderCli { param([string]$Provider) return 'Invoke-SyntheticProvider' }
    function Invoke-SyntheticProvider {
        $script:ObservedProfileEnv = [Environment]::GetEnvironmentVariable($script:ProfileVariable, 'Process')
        $global:LASTEXITCODE = $script:SyntheticExitCode
    }
    foreach ($Provider in @('claude', 'codex')) {
        $script:ProfileVariable = if ($Provider -eq 'claude') { 'CLAUDE_CONFIG_DIR' } else { 'CODEX_HOME' }
        $Previous = [Environment]::GetEnvironmentVariable($script:ProfileVariable, 'Process')
        $LoginRequest = [pscustomobject]@{ provider = $Provider; email = 'person@example.test'; directory = $ClaudeDir }
        try {
            # Modern .NET distinguishes absent and empty environment variables;
            # .NET Framework may normalize empty to absent. Preserve either.
            foreach ($Original in @($null, '', (Join-Path $TestRoot 'previous-profile'))) {
                Restore-TestEnvironment $script:ProfileVariable $Original
                $Expected = [Environment]::GetEnvironmentVariable($script:ProfileVariable, 'Process')
                foreach ($ExitCode in @(7, 0)) {
                    $script:SyntheticExitCode = $ExitCode
                    $Rejected = $false
                    try { Invoke-ProfileLogin $LoginRequest } catch { $Rejected = $true }
                    Assert-Test ($Rejected -eq ($ExitCode -ne 0)) 'Provider CLI exit code was not respected.'
                    Assert-Test ($script:ObservedProfileEnv -eq $ClaudeDir) 'Isolated process environment was not set.'
                    Assert-Test ([Environment]::GetEnvironmentVariable($script:ProfileVariable, 'Process') -ceq $Expected) 'Process environment was not restored.'
                }
            }
        } finally {
            Restore-TestEnvironment $script:ProfileVariable $Previous
        }
    }
    $Default = [pscustomobject]@{ provider = 'claude'; email = 'person@example.test'; directory = (Get-DefaultDirectory 'claude') }
    $script:ObservedProfileEnv = $null
    $Rejected = $false
    try { Invoke-ProfileLogin $Default } catch { $Rejected = $true }
    Assert-Test $Rejected 'Default login replacement was allowed.'
    Assert-Test ($null -eq $script:ObservedProfileEnv) 'Default guard invoked the CLI.'
    Write-Output 'PASS: identity, scope, expiry, activation, shared Codex homes, registry preservation, default guard, and failed-login environment restoration.'
} finally {
    $Resolved = [IO.Path]::GetFullPath($TestRoot)
    $Allowed = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if ($Resolved.StartsWith($Allowed, [StringComparison]::OrdinalIgnoreCase) -and
        (Split-Path -Leaf $Resolved).StartsWith('qhud-connection-test-')) {
        Remove-Item -LiteralPath $Resolved -Recurse -Force
    }
}
