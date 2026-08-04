[CmdletBinding()]
param(
    [ValidateSet('dark', 'light', 'custom')]
    [string]$Theme,
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'
$Generator = Join-Path $PSScriptRoot 'generate-wallpaper.ps1'
$arguments = @{
    Width = 3840
    Height = 2160
    SetWallpaper = $true
    Quiet = $Quiet
}
if ($Theme) { $arguments.Theme = $Theme }
& $Generator @arguments
