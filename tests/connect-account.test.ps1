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

    # Invoke the same helper entry point as the widget. Both apparent CLIs are
    # test shims and can write only beneath the synthetic home.
    $DefaultClaudeDir = Join-Path $TestRoot '.claude'
    $TeamCodexDir = Join-Path $TestRoot 'codex-team'
    $StageTemp = Join-Path $TestRoot 'temp'
    foreach ($Directory in @($DefaultClaudeDir, $TeamCodexDir, $StageTemp)) {
        $null = [IO.Directory]::CreateDirectory($Directory)
    }
    foreach ($Provider in @('claude', 'codex')) {
        $Shim = '@powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0provider.ps1" ' + $Provider + ' %*'
        [IO.File]::WriteAllText((Join-Path $TestRoot ($Provider + '.cmd')), $Shim, $Utf8)
    }
    [IO.File]::WriteAllText((Join-Path $TestRoot 'provider.ps1'), @'
$ErrorActionPreference = 'Stop'
$provider = $args[0]
$dir = if ($provider -eq 'claude') { $env:CLAUDE_CONFIG_DIR } else { $env:CODEX_HOME }
if (-not $dir -or -not [IO.Path]::GetFullPath($dir).StartsWith(($PSScriptRoot + '\'), [StringComparison]::OrdinalIgnoreCase)) { exit 9 }
$cliArgs = @($args | Select-Object -Skip 1)
$settings = Join-Path $dir 'settings.json'
$orgHint = if (Test-Path -LiteralPath $settings) { ([IO.File]::ReadAllText($settings) | ConvertFrom-Json).forceLoginOrgUUID }
$call = @{ dir = $dir; arguments = $cliArgs; orgHint = $orgHint }
[IO.File]::WriteAllText((Join-Path $PSScriptRoot 'call.json'), ($call | ConvertTo-Json))
$result = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'result.json')) | ConvertFrom-Json
if ($result.concurrentAuth) {
    [IO.File]::WriteAllText((Join-Path $PSScriptRoot 'claude-team\.credentials.json'), ($result.concurrentAuth | ConvertTo-Json -Depth 8))
}
if ($provider -eq 'claude') {
    [IO.File]::WriteAllText((Join-Path $dir '.claude.json'), ($result.config | ConvertTo-Json -Depth 8))
    [IO.File]::WriteAllText((Join-Path $dir '.credentials.json'), ($result.credential | ConvertTo-Json -Depth 8))
} else {
    [IO.File]::WriteAllText((Join-Path $dir 'auth.json'), ($result.auth | ConvertTo-Json -Depth 8))
}
# A failed provider may already have written a partial or different login.
exit $result.exitCode
'@, $Utf8)
    function New-TestCodexAuth($Email, $Workspace, $Plan, $Expiry) {
        return @{ tokens = @{ account_id = $Workspace
            id_token = (Test-Token @{ email = $Email; 'https://api.openai.com/auth' = @{
                chatgpt_account_id = $Workspace; chatgpt_plan_type = $Plan } })
            access_token = (Test-Token @{ exp = $Expiry }) } }
    }
    function Get-TestHashes($Paths) {
        $Hashes = @{}
        foreach ($Path in $Paths) {
            $Hashes[$Path] = if ([IO.File]::Exists($Path)) { (Get-FileHash -LiteralPath $Path).Hash } else { 'absent' }
        }
        return $Hashes
    }
    function Assert-Unchanged($Hashes) {
        $Now = Get-TestHashes @($Hashes.Keys)
        foreach ($Path in $Hashes.Keys) {
            Assert-Test ($Hashes[$Path] -ceq $Now[$Path]) ('Unrelated fixture changed: ' + $Path)
        }
    }
    $Email = 'person@example.test'
    $PersonalId = '00000000-0000-4000-a000-000000000001'
    $TeamId = '00000000-0000-4000-a000-000000000002'
    $OtherId = '00000000-0000-4000-a000-000000000003'
    $NormalExpiry = 4102444800
    $Team = [pscustomobject]@{ key='codex-isolated'; provider='codex'; email=$Email; label='Codex isolated'; scope='team'; directory=$TeamCodexDir
        seat_annotation=@{email=$Email;workspace_id=$TeamId;workspace_name='Fixture';seat_type='standard';verified_at_ms=1000} }
    $Ordinary = [pscustomobject]@{ key='claude-default'; provider='claude'; email=$Email; label='Default'; scope='personal'; directory=$DefaultClaudeDir; expected_org_id=$PersonalId }
    $Requested.key = 'claude-isolated'
    $Requested | Add-Member -NotePropertyName expected_org_id -NotePropertyValue $TeamId -Force
    $Profiles = @($Shared) + @($Team, $Ordinary, $Requested)
    Write-TestJson $script:ResolvedRegistryPath @{ profiles=$Profiles; codex_homes=@($CodexDir,$TeamCodexDir); claude_config_dirs=@($ClaudeDir); keep='unrelated' }
    $Trace = Join-Path $TestRoot 'call.json'
    $AllFiles = @($script:ResolvedRegistryPath, (Join-Path $CodexDir 'auth.json'), (Join-Path $TeamCodexDir 'auth.json'),
        (Join-Path $TestRoot '.claude.json'), (Join-Path $DefaultClaudeDir '.credentials.json'),
        (Join-Path $ClaudeDir '.claude.json'), (Join-Path $ClaudeDir '.credentials.json'))
    $SavedEnv = @{}
    foreach ($Variable in @('USERPROFILE','CODEX_HOME','CLAUDE_CONFIG_DIR','QHUD_CODEX_BIN','Path','TMP','TEMP')) {
        $SavedEnv[$Variable] = [Environment]::GetEnvironmentVariable($Variable, 'Process')
    }
    try {
        $env:USERPROFILE = $TestRoot
        $env:CODEX_HOME = $CodexDir
        $env:QHUD_CODEX_BIN = Join-Path $TestRoot 'codex.cmd'
        Restore-TestEnvironment 'CLAUDE_CONFIG_DIR' $null
        $env:Path = $TestRoot + ';' + $env:Path
        $env:TEMP = $StageTemp
        $env:TMP = $StageTemp
        $Entry = @('-NoLogo','-NoProfile','-ExecutionPolicy','Bypass','-File',$Helper,'-RegistryPath',$script:ResolvedRegistryPath)
        $Unpinned = [pscustomobject]@{key='ambiguous-team';provider='codex';label='Unpinned';email=$Email;scope='team';directory=(Join-Path $TestRoot 'ambiguous')}
        $Workspace = @{account_id=$TeamId;email=$Email;plan_type='team'}
        $Snapshot = @{ codex=@{ fetched_at_ms=1000; workspaces=@($Workspace, $Workspace,
            @{account_id=$OtherId;email='someone-else@example.test';plan_type='team'},
            @{account_id=$PersonalId;email=$Email;plan_type='pro'}) } }
        $Store = Join-Path $TestRoot 'fetched-usage.json'
        Write-TestJson $Store $Snapshot
        Assert-Test ((Get-LoginOrganization $Unpinned (Get-ProfileState $Unpinned)) -ceq $TeamId) 'A duplicate row or unrelated workspace obscured a unique target.'
        $Snapshot.codex.workspaces += @{account_id=$OtherId;email=$Email;plan_type='business'}
        Write-TestJson $Store $Snapshot
        $BeforeAmbiguity = [IO.File]::ReadAllText($script:ResolvedRegistryPath)
        Write-TestJson $script:ResolvedRegistryPath @{profiles=@($Unpinned)}
        $Untouched = Get-TestHashes @($script:ResolvedRegistryPath, (Join-Path $Unpinned.directory 'auth.json'))
        & $CurrentPowerShell @Entry -Profile $Unpinned.key -Reauthenticate | Out-Null
        Assert-Test ($LASTEXITCODE -ne 0 -and -not [IO.File]::Exists($Trace)) 'Ambiguous workspaces under the same Codex email and scope reached login.'
        Assert-Unchanged $Untouched
        [IO.File]::Delete($Store)
        [IO.File]::WriteAllText($script:ResolvedRegistryPath, $BeforeAmbiguity, $Utf8)
        $Cases = @(
            @{ profile=$Shared[0]; initial='expired' }, @{ profile=$Team; initial='expired' },
            @{ profile=$Ordinary; initial='expired' }, @{ profile=$Requested; initial='missing' },
            @{ profile=$Requested; initial='wrong_account' }
        )
        foreach ($Scenario in $Cases) {
            $Case = $Scenario.profile
            $IsClaude = $Case.provider -eq 'claude'
            $IsPersonal = $Case.scope -eq 'personal'
            $Id = if ($IsPersonal) { $PersonalId } else { $TeamId }
            $Plan = if ($IsPersonal) { 'pro' } else { 'team' }
            $OrgType = if ($IsPersonal) { 'claude_pro' } else { 'claude_team' }
            $AuthPath = Join-Path $Case.directory $(if ($IsClaude) { '.credentials.json' } else { 'auth.json' })
            $ConfigPath = if ($IsClaude) { Get-ClaudeConfigFile $Case.directory }
            $MissingClaude = $Scenario.initial -eq 'missing'
            $ReplaceClaudeIdentity = $IsClaude -and $Scenario.initial -in @('wrong_account', 'missing')
            if ($IsClaude) {
                $Config = @{ keepPreference = 'original'; oauthAccount = @{ emailAddress=$Email; accountUuid=$PersonalId; organizationUuid=$Id; organizationType=$OrgType } }
                if ($MissingClaude) {
                    $Config.Remove('oauthAccount')
                    [IO.File]::Delete($AuthPath)
                } else {
                    $Expires = 1
                    if ($Scenario.initial -eq 'wrong_account') {
                        $Config.oauthAccount.emailAddress = 'old-account@example.test'
                        $Config.oauthAccount.accountUuid = $OtherId
                        $Config.oauthAccount.organizationUuid = $OtherId
                        $Expires = 4102444800000
                    }
                    Write-TestJson $AuthPath @{ keepCredentialSetting='original'; claudeAiOauth=@{accessToken='fixture-old'; expiresAt=$Expires} }
                }
                Write-TestJson $ConfigPath $Config
            } else {
                Write-TestJson $AuthPath (New-TestCodexAuth $Email $Id $Plan 1)
            }
            if ($Case.key -eq $Shared[0].key) {
                # Batch mode and an unrelated default must not even reach the provider.
                Write-TestJson $script:ResolvedRegistryPath @{profiles=@($Shared); codex_homes=@($CodexDir)}
                & $CurrentPowerShell @Entry -AllPending | Out-Null
                Assert-Test ($LASTEXITCODE -ne 0 -and -not [IO.File]::Exists($Trace)) 'Batch mode tried the default login.'
                Write-TestJson $AuthPath (New-TestCodexAuth 'other@example.test' $OtherId 'pro' $NormalExpiry)
                & $CurrentPowerShell @Entry -Profile $Case.key -Reauthenticate | Out-Null
                Assert-Test ($LASTEXITCODE -ne 0 -and -not [IO.File]::Exists($Trace)) 'An unrelated default login reached the provider.'
                Write-TestJson $AuthPath (New-TestCodexAuth $Email $Id $Plan 1)
                Write-TestJson $script:ResolvedRegistryPath @{profiles=$Profiles; codex_homes=@($CodexDir,$TeamCodexDir); claude_config_dirs=@($ClaudeDir); keep='unrelated'}
            }
            if ($Case.key -eq $Ordinary.key) {
                [IO.File]::Delete($Trace)
                $Config.oauthAccount.organizationUuid = $OtherId
                Write-TestJson $ConfigPath $Config
                $Before = Get-TestHashes $AllFiles
                & $CurrentPowerShell @Entry -Profile $Case.key -Reauthenticate | Out-Null
                Assert-Test ($LASTEXITCODE -ne 0 -and -not [IO.File]::Exists($Trace)) 'An unrelated default Claude organization reached the provider.'
                Assert-Unchanged $Before
                $Config.oauthAccount.organizationUuid = $Id
                Write-TestJson $ConfigPath $Config
            }
            $Outcomes = @('cancel', 'other-email', 'other-workspace')
            if ($Case.key -eq $Team.key) { $Outcomes += 'other-scope' }
            if ($Scenario.initial -eq 'wrong_account') { $Outcomes += 'concurrent-update' }
            foreach ($Outcome in ($Outcomes + 'match')) {
                [IO.File]::Delete($Trace)
                $ProviderEmail = if ($Outcome -eq 'other-email') { 'other@example.test' } else { $Email }
                $ProviderId = if ($Outcome -eq 'other-workspace') { $OtherId } else { $Id }
                $Result = @{ exitCode=$(if ($Outcome -eq 'cancel') { 7 } else { 0 }) }
                if ($Outcome -eq 'concurrent-update') {
                    $Result.concurrentAuth = @{ keepCredentialSetting='original'; independentEdit='keep'; claudeAiOauth=@{accessToken='fixture-concurrent';expiresAt=4102444800000} }
                }
                if ($IsClaude) {
                    $Result.config=@{ cliOnly='stage-only';oauthAccount=@{emailAddress=$ProviderEmail;accountUuid=$PersonalId;organizationUuid=$ProviderId;organizationType=$OrgType} }
                    $Result.credential=@{cliOnly='stage-only';claudeAiOauth=@{accessToken='fixture-fresh';expiresAt=4102444800000}}
                } else {
                    $ResultPlan = if ($Outcome -eq 'other-scope') { 'pro' } else { $Plan }
                    $Result.auth = New-TestCodexAuth $ProviderEmail $ProviderId $ResultPlan $NormalExpiry
                }
                Write-TestJson (Join-Path $TestRoot 'result.json') $Result
                $Protected = @($AllFiles)
                if ($Outcome -eq 'match') {
                    $Protected = @($Protected | Where-Object { $_ -ne $AuthPath -and (-not $ReplaceClaudeIdentity -or $_ -ne $ConfigPath) })
                } elseif ($Outcome -eq 'concurrent-update') {
                    $Protected = @($Protected | Where-Object { $_ -ne $AuthPath })
                }
                $Before = Get-TestHashes $Protected
                & $CurrentPowerShell @Entry -Profile $Case.key -Reauthenticate | Out-Null
                $Code = $LASTEXITCODE
                $Expected = switch ($Outcome) { 'cancel' {1} 'other-email' {3} 'other-workspace' {4} 'other-scope' {4} 'concurrent-update' {1} 'match' {0} }
                Assert-Test ($Code -eq $Expected) ("$($Case.key)/$($Scenario.initial)/$Outcome returned $Code, expected $Expected.")
                Assert-Unchanged $Before
                if ($Outcome -eq 'concurrent-update') {
                    Assert-Test ((Read-LocalJson $AuthPath).independentEdit -eq 'keep') 'Concurrent CLI update was lost.'
                }
                $Call = Read-LocalJson $Trace
                Assert-Test ($null -ne $Call -and $Call.dir -ne $Case.directory) 'Provider ran in the original profile.'
                Assert-Test (-not [IO.Directory]::Exists($Call.dir)) 'Staged provider credentials were left behind.'
                if ($IsClaude) {
                    Assert-Test (($Call.arguments -join ' ') -match ('--email ' + [Regex]::Escape($Email))) 'Claude login omitted the target email.'
                    Assert-Test ($Call.orgHint -ceq $Id) 'Claude login omitted the target organization setting.'
                } else {
                    Assert-Test (($Call.arguments -join ' ') -match ('forced_chatgpt_workspace_id="?'+$Id+'"?')) 'Codex login omitted the target workspace setting.'
                }
            }
            Assert-Test ((Get-ProfileState $Case).State -eq 'ready') 'The matching staged login was not installed.'
            if ($IsClaude) {
                $FinalConfig = Read-LocalJson $ConfigPath
                Assert-Test ($FinalConfig.keepPreference -eq 'original' -and -not (Get-PropertyValue $FinalConfig 'cliOnly')) 'Claude settings were replaced by staging settings.'
                if (-not $MissingClaude) {
                    $FinalAuth = Read-LocalJson $AuthPath
                    Assert-Test ($FinalAuth.keepCredentialSetting -eq 'original' -and -not (Get-PropertyValue $FinalAuth 'cliOnly')) 'Other Claude credential settings were replaced.'
                }
            }
        }
    } finally {
        foreach ($Variable in $SavedEnv.Keys) { Restore-TestEnvironment $Variable $SavedEnv[$Variable] }
    }

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
                    try { Invoke-ProviderLogin $LoginRequest $ClaudeDir $null } catch { $Rejected = $true }
                    Assert-Test ($Rejected -eq ($ExitCode -ne 0)) 'Provider CLI exit code was not respected.'
                    Assert-Test ($script:ObservedProfileEnv -eq $ClaudeDir) 'Isolated process environment was not set.'
                    Assert-Test ([Environment]::GetEnvironmentVariable($script:ProfileVariable, 'Process') -ceq $Expected) 'Process environment was not restored.'
                }
            }
        } finally {
            Restore-TestEnvironment $script:ProfileVariable $Previous
        }
    }
    Write-Output 'PASS: targeted CLI hints, guarded default and isolated logins, failure isolation, verified activation, registry preservation, and environment restoration.'
} finally {
    $Resolved = [IO.Path]::GetFullPath($TestRoot)
    $Allowed = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if ($Resolved.StartsWith($Allowed, [StringComparison]::OrdinalIgnoreCase) -and
        (Split-Path -Leaf $Resolved).StartsWith('qhud-connection-test-')) {
        Remove-Item -LiteralPath $Resolved -Recurse -Force
    }
}
