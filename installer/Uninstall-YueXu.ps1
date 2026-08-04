[CmdletBinding()]
param()

$ErrorActionPreference = 'Continue'
$InstallRoot = Split-Path -Parent $PSCommandPath
$TaskName = 'YueXuWallpaper'

foreach ($task in @($TaskName, 'LunarCalendarDailyWallpaper')) {
    if (Get-ScheduledTask -TaskName $task -ErrorAction SilentlyContinue) {
        Unregister-ScheduledTask -TaskName $task -Confirm:$false
    }
}

$desktopShortcut = Join-Path ([Environment]::GetFolderPath('Desktop')) '月序日历.lnk'
if (Test-Path -LiteralPath $desktopShortcut) { Remove-Item -LiteralPath $desktopShortcut -Force }

$startMenuFolder = Join-Path ([Environment]::GetFolderPath('StartMenu')) 'Programs\月序'
if (Test-Path -LiteralPath $startMenuFolder) { Remove-Item -LiteralPath $startMenuFolder -Recurse -Force }

Remove-Item -LiteralPath 'HKCU:\Software\Classes\yuexu' -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\YueXu' -Recurse -Force -ErrorAction SilentlyContinue

# 保留当前壁纸与用户主题偏好，防止卸载后桌面突然变成纯色。
Set-Location $env:TEMP
if (Test-Path -LiteralPath $InstallRoot) { Remove-Item -LiteralPath $InstallRoot -Recurse -Force }

Write-Output '月序已卸载。当前壁纸和用户数据已保留。'
