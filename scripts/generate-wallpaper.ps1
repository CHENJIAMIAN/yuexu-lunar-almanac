[CmdletBinding()]
param(
    [int]$Width = 3840,
    [int]$Height = 2160,
    [int]$Year = (Get-Date).Year,
    [ValidateSet('dark', 'light')]
    [string]$Theme = 'dark',
    [switch]$Quiet,
    [switch]$SetWallpaper
)

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$IndexPath = Join-Path $ProjectRoot 'index.html'
$OutputPath = Join-Path $ProjectRoot 'output\lunar-wallpaper.png'

function Find-Chrome {
    $candidates = @(
        (Get-Command chrome.exe -ErrorAction SilentlyContinue).Source,
        (Get-Command msedge.exe -ErrorAction SilentlyContinue).Source,
        'C:\Program Files\Google\Chrome\Application\chrome.exe',
        'C:\Program Files (x86)\Google\Chrome\Application\chrome.exe',
        'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe',
        'C:\Program Files\Microsoft\Edge\Application\msedge.exe'
    ) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -Unique
    if (-not $candidates) { throw '未找到 Chrome 或 Edge。请安装任一 Chromium 浏览器后重试。' }
    return $candidates[0]
}

function Set-DesktopWallpaper([string]$Path) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class LunarWallpaper {
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern bool SystemParametersInfo(uint action, uint param, string value, uint flags);
}
'@ -ErrorAction SilentlyContinue
    $ok = [LunarWallpaper]::SystemParametersInfo(20, 0, $Path, 3)
    if (-not $ok) { throw "Windows 拒绝设置壁纸：$([Runtime.InteropServices.Marshal]::GetLastWin32Error())" }
}

if (-not (Test-Path $IndexPath)) { throw "找不到日历页面：$IndexPath" }
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutputPath) | Out-Null

$chrome = Find-Chrome
$renderToken = [DateTime]::UtcNow.Ticks
$ProfilePath = Join-Path $ProjectRoot "output\headless-profile-$renderToken"
$profileRoot = (Resolve-Path (Split-Path -Parent $ProfilePath)).Path
if (-not $ProfilePath.StartsWith($profileRoot, [StringComparison]::OrdinalIgnoreCase)) { throw '无效的浏览器临时目录。' }
New-Item -ItemType Directory -Force -Path $ProfilePath | Out-Null
$indexUrl = 'file:///' + ($IndexPath -replace '\\', '/') + "?wallpaper=1&year=$Year&theme=$Theme&width=$Width&height=$Height&render=$renderToken"
$chromeArgs = @(
    '--headless=new',
    '--disable-gpu',
    '--hide-scrollbars',
    '--disable-background-networking',
    '--disable-extensions',
    "--user-data-dir=$ProfilePath",
    '--force-device-scale-factor=1',
    "--window-size=$Width,$Height",
    "--screenshot=$OutputPath",
    $indexUrl
)

try {
    if (Test-Path $OutputPath) { Remove-Item -LiteralPath $OutputPath -Force }
    $process = Start-Process -FilePath $chrome -ArgumentList $chromeArgs -Wait -PassThru -WindowStyle Hidden
    if ($process.ExitCode -ne 0 -or -not (Test-Path $OutputPath)) {
        throw "壁纸渲染失败，浏览器退出码：$($process.ExitCode)"
    }
} finally {
    if (Test-Path $ProfilePath) { Remove-Item -LiteralPath $ProfilePath -Recurse -Force -ErrorAction SilentlyContinue }
}

if ($SetWallpaper) { Set-DesktopWallpaper $OutputPath }

if (-not $Quiet) {
    $size = (Get-Item $OutputPath).Length
    Write-Output "已生成：$OutputPath"
    Write-Output "尺寸：${Width}×${Height}，文件：$([math]::Round($size / 1KB, 1)) KB"
    if ($SetWallpaper) { Write-Output '已设置为 Windows 桌面背景。' }
}
