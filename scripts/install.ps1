[CmdletBinding()]
param(
    [ValidateSet('keep', 'dark', 'light')]
    [string]$Theme = 'keep'
)

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$Version = (Get-Content -LiteralPath (Join-Path $ProjectRoot 'VERSION') -Raw -Encoding UTF8).Trim()
$buildScript = Join-Path $PSScriptRoot 'Build-Release.ps1'
& $buildScript -Version $Version -SkipArchive
$stage = Join-Path $ProjectRoot "dist\YueXu-$Version-windows-x64"
& (Join-Path $stage 'Install-YueXu.ps1') -Theme $Theme
