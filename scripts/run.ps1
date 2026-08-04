[CmdletBinding()]
param(
    [ValidateSet('dark', 'light')]
    [string]$Theme = 'dark',
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$Generator = Join-Path $PSScriptRoot 'generate-wallpaper.ps1'

& $Generator -Width 3840 -Height 2160 -Theme $Theme -SetWallpaper:$true -Quiet:$Quiet
