[CmdletBinding()]
param(
    [ValidateSet('dark', 'light')]
    [string]$Theme = 'dark'
)

$ErrorActionPreference = 'Stop'
$PackageRoot = Split-Path -Parent $PSCommandPath
$SourceBinary = Join-Path $PackageRoot 'LunarCalendar.exe'
$SourceUninstaller = Join-Path $PackageRoot 'Uninstall-YueXu.ps1'
$SourceIcon = Join-Path $PackageRoot 'YueXu.ico'
$VersionFile = Join-Path $PackageRoot 'VERSION'
$InstallRoot = Join-Path $env:LOCALAPPDATA 'Programs\YueXu'
$TaskName = 'YueXuWallpaper'
$CodexTaskPath = '\Codex\'
$LegacyTaskPath = '\'

if (-not (Test-Path -LiteralPath $SourceBinary)) { throw "找不到发行程序：$SourceBinary" }
if (-not (Test-Path -LiteralPath $SourceUninstaller)) { throw "找不到卸载程序：$SourceUninstaller" }
if (-not (Test-Path -LiteralPath $SourceIcon)) { throw "找不到产品图标：$SourceIcon" }

function Ensure-CodexTaskFolder {
    $service = New-Object -ComObject 'Schedule.Service'
    $service.Connect()
    $root = $service.GetFolder('\')
    try {
        $root.GetFolder('Codex') | Out-Null
    } catch {
        $root.CreateFolder('Codex', $null) | Out-Null
    }
}

Ensure-CodexTaskFolder

$Version = if (Test-Path -LiteralPath $VersionFile) { (Get-Content -LiteralPath $VersionFile -Raw -Encoding UTF8).Trim() } else { '0.1.0' }
$Binary = Join-Path $InstallRoot 'LunarCalendar.exe'
$Uninstaller = Join-Path $InstallRoot 'Uninstall-YueXu.ps1'
$Icon = Join-Path $InstallRoot 'YueXu.ico'

New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null
Copy-Item -LiteralPath $SourceBinary -Destination $Binary -Force
Copy-Item -LiteralPath $SourceUninstaller -Destination $Uninstaller -Force
Copy-Item -LiteralPath $SourceIcon -Destination $Icon -Force

# 用新版本先生成一次，确保安装完成时桌面就是可用状态。
$initialUpdate = Start-Process -FilePath $Binary -ArgumentList @('--update', '--theme', $Theme, '--quiet') -Wait -PassThru -WindowStyle Hidden
if ($initialUpdate.ExitCode -ne 0) { throw "月序无法生成初始壁纸，退出码：$($initialUpdate.ExitCode)" }

foreach ($legacyTask in @('LunarCalendarDailyWallpaper', $TaskName)) {
    $existing = Get-ScheduledTask -TaskPath $LegacyTaskPath -TaskName $legacyTask -ErrorAction SilentlyContinue
    if ($existing) { Unregister-ScheduledTask -TaskPath $LegacyTaskPath -TaskName $legacyTask -Confirm:$false }
}

$action = New-ScheduledTaskAction -Execute $Binary -Argument '--update --quiet'
$triggers = @(
    (New-ScheduledTaskTrigger -AtLogOn),
    (New-ScheduledTaskTrigger -Daily -At 00:01)
)
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -ExecutionTimeLimit (New-TimeSpan -Minutes 3)
Register-ScheduledTask -TaskPath $CodexTaskPath -TaskName $TaskName -Action $action -Trigger $triggers -Settings $settings -Description '月序：每天更新农历桌面日历。' -Force | Out-Null

function New-YueXuShortcut {
    param(
        [Parameter(Mandatory)] [string]$Path,
        [Parameter(Mandatory)] [string]$Target,
        [Parameter(Mandatory)] [string]$Arguments,
        [Parameter(Mandatory)] [string]$IconPath
    )

    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($Path)
    $shortcut.TargetPath = $Target
    $shortcut.Arguments = $Arguments
    $shortcut.WorkingDirectory = Split-Path -Parent $Target
    $shortcut.Description = '设置月序农历桌面日历'
    $shortcut.IconLocation = "$IconPath,0"
    $shortcut.Save()
}

$desktop = [Environment]::GetFolderPath('Desktop')
New-YueXuShortcut -Path (Join-Path $desktop '月序日历.lnk') -Target $Binary -Arguments '--preview' -IconPath $Icon

$programs = Join-Path ([Environment]::GetFolderPath('StartMenu')) 'Programs\月序'
New-Item -ItemType Directory -Force -Path $programs | Out-Null
New-YueXuShortcut -Path (Join-Path $programs '月序日历.lnk') -Target $Binary -Arguments '--preview' -IconPath $Icon

$protocolKey = 'HKCU:\Software\Classes\yuexu'
New-Item -Path "$protocolKey\shell\open\command" -Force | Out-Null
Set-Item -Path $protocolKey -Value 'URL:月序 Protocol'
New-ItemProperty -Path $protocolKey -Name 'URL Protocol' -Value '' -PropertyType String -Force | Out-Null
Set-Item -Path "$protocolKey\shell\open\command" -Value "`"$Binary`" `"%1`""

$pwsh = (Get-Command pwsh.exe -ErrorAction SilentlyContinue).Source
if (-not $pwsh) { $pwsh = (Get-Command powershell.exe -ErrorAction Stop).Source }
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\YueXu'
New-Item -Path $uninstallKey -Force | Out-Null
$uninstallValues = @{
    DisplayName = '月序 / Lunar Almanac'
    DisplayVersion = $Version
    Publisher = '月序'
    DisplayIcon = $Icon
    InstallLocation = $InstallRoot
    UninstallString = "`"$pwsh`" -NoProfile -ExecutionPolicy Bypass -File `"$Uninstaller`""
    NoModify = 1
    NoRepair = 1
}
foreach ($entry in $uninstallValues.GetEnumerator()) {
    New-ItemProperty -Path $uninstallKey -Name $entry.Key -Value $entry.Value -Force | Out-Null
}

Write-Output '月序已安装。'
Write-Output "安装目录：$InstallRoot"
Write-Output "自动更新任务：$CodexTaskPath$TaskName"
