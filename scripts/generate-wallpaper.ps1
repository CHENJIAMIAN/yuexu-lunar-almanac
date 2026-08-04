[CmdletBinding()]
param(
    [int]$Width = 3840,
    [int]$Height = 2160,
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

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutputPath) | Out-Null
if (-not (Test-Path -LiteralPath $Binary)) {
    Push-Location $ProjectRoot
    try { cargo build --bin LunarCalendar } finally { Pop-Location }
}

$arguments = @(
    '--update',
    "--width=$Width",
    "--height=$Height",
    "--year=$Year",
    "--output=$OutputPath",
    "--set-wallpaper=$($SetWallpaper.IsPresent.ToString().ToLowerInvariant())",
    '--quiet'
)
if ($Theme) { $arguments += "--theme=$Theme" }

& $Binary @arguments
if ($LASTEXITCODE -ne 0) { throw "Rust 壁纸渲染器退出码：$LASTEXITCODE" }

if (-not $Quiet) {
    $size = (Get-Item $OutputPath).Length
    Write-Output "已生成：$OutputPath"
    Write-Output "尺寸：${Width}×${Height}，文件：$([math]::Round($size / 1KB, 1)) KB"
    if ($SetWallpaper) { Write-Output '已设置为 Windows 桌面背景。' }
}
