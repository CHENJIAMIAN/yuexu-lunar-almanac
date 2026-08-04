[CmdletBinding()]
param()

$ErrorActionPreference = 'SilentlyContinue'
$taskName = 'LunarCalendarDailyWallpaper'
$taskPaths = @('\Codex\', '\')
foreach ($taskPath in $taskPaths) {
    Unregister-ScheduledTask -TaskPath $taskPath -TaskName $taskName -Confirm:$false
}
$shortcut = Join-Path ([Environment]::GetFolderPath('Desktop')) '月序日历.lnk'
if (Test-Path $shortcut) { Remove-Item -LiteralPath $shortcut -Force }
Write-Output "已移除任务和快捷方式：\Codex\$taskName"
