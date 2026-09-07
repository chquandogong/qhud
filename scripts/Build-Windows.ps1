#requires -Version 5.1
<#
.SYNOPSIS
Build QHUD for native Windows x64 with its pinned Qmonster dependency.
.DESCRIPTION
Uses Git and Cargo from PATH first. Cargo may also be found under the adjacent
tools/rust/cargo/bin or tools/cargo/bin directory. Does not install toolchains,
overwrite a modified dependency, or configure automatic startup.
Use -Test to run QHUD tests before compilation. The Windows dependency patch
applies only to these Cargo commands; Cargo.lock is restored byte for byte.
#>
[CmdletBinding()]
param(
    [string]$ToolsRoot,
    [string]$VisualStudioPath,
    [switch]$DebugBuild,
    [switch]$Test,
    [switch]$PrepareOnly,
    [switch]$Run
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false

if ($env:OS -ne 'Windows_NT') {
    throw 'This script builds the native Windows x64 application. Run it on Windows.'
}
if ($PrepareOnly -and ($Run -or $Test)) {
    throw '-PrepareOnly cannot be combined with -Run or -Test.'
}

$ProjectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$WorkspaceRoot = Split-Path -Parent $ProjectRoot
$DependenciesRoot = Join-Path $WorkspaceRoot 'qhud-deps'
$DependencyPath = Join-Path $DependenciesRoot 'qmonster'
$PatchPath = Join-Path $ProjectRoot 'patches\qmonster-windows.patch'
$ManifestPath = Join-Path $ProjectRoot 'src-tauri\Cargo.toml'
$PinnedRevision = '6a21c447f41dd366e6ea7e08fb3e8545a03396d5'
$DependencyUrl = 'https://github.com/chquandogong/qmonster'
$TargetTriple = 'x86_64-pc-windows-msvc'

if (-not (Test-Path -LiteralPath $ManifestPath -PathType Leaf) -or
    -not (Test-Path -LiteralPath $PatchPath -PathType Leaf)) {
    throw 'Run the checked-in script from the QHUD source tree; its manifest and patch are required.'
}
if (-not $ToolsRoot) {
    $ToolsRoot = Join-Path $WorkspaceRoot 'tools'
}
$ToolsRoot = [IO.Path]::GetFullPath($ToolsRoot)

function Assert-OrdinaryDirectory {
    param([string]$Path)
    if (Test-Path -LiteralPath $Path) {
        $Item = Get-Item -LiteralPath $Path -Force
        if (-not $Item.PSIsContainer -or
            ($Item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw "Refusing to modify a file, symlink, or junction at dependency path: $Path"
        }
    }
}

$GitCommand = Get-Command git.exe -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $GitCommand) {
    throw 'Git for Windows must be available on PATH. See docs/05-ops/WINDOWS.md.'
}
$GitExe = $GitCommand.Source

function Invoke-Git {
    param([string[]]$Arguments)
    & $GitExe @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Git failed with exit code $LASTEXITCODE. Dependency files have not been reset or deleted."
    }
}

function Test-DependencyPatchApplied {
    # A failed reverse check is expected for a clean checkout. Suppress its
    # diagnostic without turning it into a terminating native error on PS 5.1.
    $ErrorActionPreference = 'Continue'
    & $GitExe @RepoGitArguments apply --reverse --check $PatchPath 2>&1 | Out-Null
    return ($LASTEXITCODE -eq 0)
}

# Respect an existing Rust setup. The fallback only examines these explicit
# workspace paths; it does not search unrelated user or system directories.
$CargoCommand = Get-Command cargo.exe -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if ($CargoCommand) {
    $CargoExe = $CargoCommand.Source
} else {
    $RustLayouts = @(
        @{ Cargo = (Join-Path $ToolsRoot 'rust\cargo'); Rustup = (Join-Path $ToolsRoot 'rust\rustup') },
        @{ Cargo = (Join-Path $ToolsRoot 'cargo'); Rustup = (Join-Path $ToolsRoot 'rustup') }
    )
    $CargoExe = $null
    foreach ($Layout in $RustLayouts) {
        $Candidate = Join-Path $Layout.Cargo 'bin\cargo.exe'
        if (Test-Path -LiteralPath $Candidate -PathType Leaf) {
            $CargoExe = $Candidate
            $env:CARGO_HOME = $Layout.Cargo
            if (Test-Path -LiteralPath $Layout.Rustup -PathType Container) {
                $env:RUSTUP_HOME = $Layout.Rustup
            }
            $env:PATH = "$(Split-Path -Parent $Candidate);$env:PATH"
            break
        }
    }
    if (-not $CargoExe) {
        throw 'Rust 1.88+ with the Windows MSVC toolchain is required. See docs/05-ops/WINDOWS.md.'
    }
}

# An existing Developer PowerShell environment takes precedence. Otherwise use
# the explicit VS path or the local tools/vs-buildtools installation.
if (-not (Get-Command cl.exe -CommandType Application -ErrorAction SilentlyContinue)) {
    if (-not $VisualStudioPath) {
        $VisualStudioPath = Join-Path $ToolsRoot 'vs-buildtools'
    }
    $VisualStudioPath = [IO.Path]::GetFullPath($VisualStudioPath)
    $DevShellModule = Join-Path $VisualStudioPath 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll'
    if (-not (Test-Path -LiteralPath $DevShellModule -PathType Leaf)) {
        throw 'Open Developer PowerShell for Visual Studio, or pass -VisualStudioPath with C++ build tools and a Windows SDK installed.'
    }
    Import-Module -Name $DevShellModule -ErrorAction Stop
    Enter-VsDevShell -VsInstallPath $VisualStudioPath -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
if (-not (Get-Command cl.exe -CommandType Application -ErrorAction SilentlyContinue)) {
    throw 'The Visual Studio C++ compiler is unavailable. Install the Desktop development with C++ workload.'
}

Assert-OrdinaryDirectory $DependenciesRoot
Assert-OrdinaryDirectory $DependencyPath
if (-not (Test-Path -LiteralPath $DependencyPath)) {
    New-Item -ItemType Directory -Path $DependenciesRoot -Force | Out-Null
    Write-Host "Cloning pinned Qmonster dependency into $DependencyPath"
    Invoke-Git -Arguments @('clone', '--no-checkout', $DependencyUrl, $DependencyPath)
    Invoke-Git -Arguments @('-C', $DependencyPath, 'checkout', '--detach', $PinnedRevision)
}
if (-not (Test-Path -LiteralPath (Join-Path $DependencyPath '.git'))) {
    throw "The dependency directory exists without Git metadata. Preserve it and choose a separate workspace: $DependencyPath"
}

# The safe-directory exception is restricted to this designated dependency and
# this invocation; no global Git configuration is changed.
$RepoGitArguments = @('-c', "safe.directory=$DependencyPath", '-C', $DependencyPath)
$Head = [string](Invoke-Git -Arguments ($RepoGitArguments + @('rev-parse', 'HEAD')))
$Dirty = @(Invoke-Git -Arguments ($RepoGitArguments + @('status', '--porcelain', '--untracked-files=all')))
if ($Head.Trim() -ne $PinnedRevision) {
    if ($Dirty.Count -gt 0) {
        throw 'Qmonster has local changes at a different revision. Preserve or commit them before rerunning; this script will not overwrite them.'
    }
    Invoke-Git -Arguments ($RepoGitArguments + @('fetch', 'origin', $PinnedRevision))
    Invoke-Git -Arguments ($RepoGitArguments + @('checkout', '--no-overwrite-ignore', '--detach', $PinnedRevision))
}

if (Test-DependencyPatchApplied) {
    Write-Host 'Qmonster Windows patch is already applied; existing source changes are preserved.'
} else {
    $Dirty = @(Invoke-Git -Arguments ($RepoGitArguments + @('status', '--porcelain', '--untracked-files=all')))
    if ($Dirty.Count -gt 0) {
        throw 'Qmonster contains local changes and does not match the complete Windows patch. No patch was applied; preserve the changes before preparing a clean dependency.'
    }
    Invoke-Git -Arguments ($RepoGitArguments + @('apply', '--check', $PatchPath))
    Invoke-Git -Arguments ($RepoGitArguments + @('apply', $PatchPath))
    Write-Host 'Applied patches/qmonster-windows.patch.'
}

& $CargoExe --version
if ($LASTEXITCODE -ne 0) {
    throw 'Cargo is not usable with the selected Rust environment.'
}
if ($PrepareOnly) {
    Write-Host 'Dependency and build environment are ready; compilation was not requested.'
    return
}

$TargetDirectory = Join-Path $ProjectRoot 'target'
$LockPath = Join-Path $ProjectRoot 'Cargo.lock'
$OriginalLockBytes = [IO.File]::ReadAllBytes($LockPath)
$Utf8 = New-Object System.Text.UTF8Encoding($false, $true)
$LockText = $Utf8.GetString($OriginalLockBytes)
$PinnedSourceLine = 'source = "git+' + $DependencyUrl + '?rev=' + $PinnedRevision + '#' + $PinnedRevision + '"'
$PackagePattern = '(?<header>\[\[package\]\]\r?\nname = "qmonster"\r?\nversion = "3\.2\.0"\r?\n)' +
    [regex]::Escape($PinnedSourceLine) + '\r?\n'
$PackageRegex = New-Object System.Text.RegularExpressions.Regex($PackagePattern)
if ($PackageRegex.Matches($LockText).Count -ne 1) {
    throw 'Cargo.lock must contain exactly one Qmonster 3.2.0 entry from the pinned Git revision. No lockfile changes were made.'
}
$WindowsLockText = $PackageRegex.Replace($LockText, '${header}', 1)

# A command-scoped config leaves ordinary Linux cargo builds on the pinned
# Git dependency. Use a temporary file to preserve TOML quotes on PowerShell
# 5.1 as well as 7, including when the checkout path contains spaces.
$CargoConfigPath = Join-Path ([IO.Path]::GetTempPath()) ('qhud-windows-' + [guid]::NewGuid().ToString('N') + '.toml')
$DependencyConfigPath = ConvertTo-Json -InputObject ($DependencyPath.Replace('\', '/')) -Compress
$CargoConfigText = '[patch."' + $DependencyUrl + '"]' + [Environment]::NewLine +
    'qmonster = { path = ' + $DependencyConfigPath + ' }' + [Environment]::NewLine
$CargoArguments = @('--config', $CargoConfigPath, '--locked', '--manifest-path', $ManifestPath,
    '--target', $TargetTriple, '--target-dir', $TargetDirectory, '-p', 'qhud')
$ProfileName = 'debug'
if (-not $DebugBuild) {
    $CargoArguments += '--release'
    $ProfileName = 'release'
}
Push-Location -LiteralPath $ProjectRoot
try {
    [IO.File]::WriteAllText($CargoConfigPath, $CargoConfigText, $Utf8)
    # A path dependency has the same lock entry except for its Git source.
    # Change only that exact validated line for this build and restore all
    # original bytes even when tests or compilation fail.
    [IO.File]::WriteAllText($LockPath, $WindowsLockText, $Utf8)
    if ($Test) {
        & $CargoExe test @CargoArguments
        if ($LASTEXITCODE -ne 0) {
            throw "QHUD tests failed with exit code $LASTEXITCODE."
        }
    }
    & $CargoExe build @CargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "QHUD compilation failed with exit code $LASTEXITCODE."
    }
} finally {
    try {
        [IO.File]::WriteAllBytes($LockPath, $OriginalLockBytes)
    } finally {
        Pop-Location
        if (Test-Path -LiteralPath $CargoConfigPath -PathType Leaf) {
            Remove-Item -LiteralPath $CargoConfigPath -Force
        }
    }
}
$Executable = Join-Path $TargetDirectory "$TargetTriple\$ProfileName\qhud.exe"
if (-not (Test-Path -LiteralPath $Executable -PathType Leaf)) {
    throw "Cargo succeeded, but the expected executable was not found: $Executable"
}
Write-Host "Built QHUD: $Executable"
if ($Run) {
    Start-Process -FilePath $Executable -WorkingDirectory $ProjectRoot -WindowStyle Hidden
}
