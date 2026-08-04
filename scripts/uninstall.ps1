[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$installRoot = Join-Path $env:LOCALAPPDATA 'Programs\YueXu'
$uninstaller = Join-Path $installRoot 'Uninstall-YueXu.ps1'
if (-not (Test-Path -LiteralPath $uninstaller)) {
    Write-Output '未找到已安装的月序。'
    exit 0
}
$pwsh = (Get-Command pwsh.exe -ErrorAction SilentlyContinue).Source
if (-not $pwsh) { $pwsh = (Get-Command powershell.exe -ErrorAction Stop).Source }
& $pwsh -NoProfile -ExecutionPolicy Bypass -File $uninstaller
