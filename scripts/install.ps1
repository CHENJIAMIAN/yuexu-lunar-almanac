[CmdletBinding()]
param(
    [switch]$SkipDesktopShortcut
)

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$RunScript = Join-Path $PSScriptRoot 'run.ps1'
$TaskName = 'LunarCalendarDailyWallpaper'
$CodexTaskPath = '\Codex\'
$LegacyTaskPath = '\'

if (-not (Test-Path $RunScript)) { throw "找不到启动脚本：$RunScript" }

function Ensure-TaskFolder([string]$FolderName) {
    $service = New-Object -ComObject 'Schedule.Service'
    $service.Connect()
    $root = $service.GetFolder('\')
    try {
        $root.GetFolder($FolderName) | Out-Null
    } catch {
        $root.CreateFolder($FolderName, $null) | Out-Null
    }
}

Ensure-TaskFolder 'Codex'

& $RunScript

$pwsh = (Get-Command pwsh.exe -ErrorAction SilentlyContinue).Source
if (-not $pwsh) { $pwsh = (Get-Command powershell.exe).Source }
$action = New-ScheduledTaskAction -Execute $pwsh -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$RunScript`" -Quiet"
$triggers = @(
    (New-ScheduledTaskTrigger -AtLogOn),
    (New-ScheduledTaskTrigger -Daily -At 00:01)
)
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -ExecutionTimeLimit (New-TimeSpan -Minutes 3)
$legacyTask = Get-ScheduledTask -TaskName $TaskName -TaskPath $LegacyTaskPath -ErrorAction SilentlyContinue
Register-ScheduledTask -TaskPath $CodexTaskPath -TaskName $TaskName -Action $action -Trigger $triggers -Settings $settings -Description '月序：每天更新当天高亮的农历桌面日历。' -Force | Out-Null
if ($legacyTask) {
    Unregister-ScheduledTask -TaskPath $LegacyTaskPath -TaskName $TaskName -Confirm:$false
}

if (-not $SkipDesktopShortcut) {
    $desktop = [Environment]::GetFolderPath('Desktop')
    $shortcutPath = Join-Path $desktop '月序日历.lnk'
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = $pwsh
    $shortcut.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$RunScript`""
    $shortcut.WorkingDirectory = $ProjectRoot
    $shortcut.Description = '生成并设置月序农历桌面日历'
    $shortcut.IconLocation = "$env:SystemRoot\System32\imageres.dll,109"
    $shortcut.Save()
    Write-Output "桌面快捷方式：$shortcutPath"
}

Write-Output "已安装任务：$CodexTaskPath$TaskName"
Write-Output '以后登录 Windows 或每天 00:01 会自动更新。'
