[CmdletBinding()]
param(
    [int]$Year = (Get-Date).Year,
    [ValidateSet('dark', 'light')]
    [string]$Theme = 'dark'
)

$ProjectRoot = Split-Path -Parent $PSScriptRoot
$index = Join-Path $ProjectRoot 'index.html'
$url = 'file:///' + ($index -replace '\\', '/') + "?year=$Year&theme=$Theme"
Start-Process $url
