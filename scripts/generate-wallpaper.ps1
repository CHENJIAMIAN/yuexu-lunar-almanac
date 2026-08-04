[CmdletBinding()]
param(
    [int]$Width = 0,
    [int]$Height = 0,
    [int]$Year = (Get-Date).Year,
    [ValidateSet('dark', 'light', 'custom')]
    [string]$Theme,
    [switch]$Quiet,
    [switch]$SetWallpaper
)

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$Binary = Join-Path $ProjectRoot 'target\debug\LunarCalendar.exe'
$OutputPath = Join-Path $ProjectRoot 'output\lunar-wallpaper.png'

if (($Width -gt 0) -xor ($Height -gt 0)) {
    throw 'Width 和 Height 必须同时指定，或同时省略以使用当前主屏分辨率。'
}
if ($Width -gt 0 -and ($Width -lt 800 -or $Height -lt 600)) {
    throw '壁纸尺寸至少需要 800×600。'
}

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutputPath) | Out-Null
if (-not (Test-Path -LiteralPath $Binary)) {
    Push-Location $ProjectRoot
    try { cargo build --bin LunarCalendar } finally { Pop-Location }
}

$arguments = @(
    '--update',
    "--year=$Year",
    "--output=$OutputPath",
    "--set-wallpaper=$($SetWallpaper.IsPresent.ToString().ToLowerInvariant())",
    '--quiet'
)
if ($Width -gt 0) {
    $arguments += @("--width=$Width", "--height=$Height")
}
if ($Theme) { $arguments += "--theme=$Theme" }

$process = Start-Process -FilePath $Binary -ArgumentList $arguments -WorkingDirectory $ProjectRoot -Wait -PassThru -WindowStyle Hidden
if ($process.ExitCode -ne 0) { throw "Rust 壁纸渲染器退出码：$($process.ExitCode)" }

if (-not $Quiet) {
    $size = (Get-Item $OutputPath).Length
    Write-Output "已生成：$OutputPath"
    $resolution = if ($Width -gt 0) { "${Width}×${Height}" } else { '当前主屏分辨率' }
    Write-Output "尺寸：$resolution，文件：$([math]::Round($size / 1KB, 1)) KB"
    if ($SetWallpaper) { Write-Output '已设置为 Windows 桌面背景。' }
}
