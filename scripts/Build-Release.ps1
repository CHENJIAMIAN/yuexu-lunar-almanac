[CmdletBinding()]
param(
    [string]$Version = (Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'VERSION') -Raw -Encoding UTF8).Trim(),
    [switch]$SkipArchive
)

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$DistRoot = Join-Path $ProjectRoot 'dist'
$ReleaseRoot = Join-Path $ProjectRoot 'release'
$StageRoot = Join-Path $DistRoot "YueXu-$Version-windows-x64"
$ArchivePath = Join-Path $ReleaseRoot "YueXu-$Version-windows-x64.zip"

if ($Version -notmatch '^\d+\.\d+\.\d+([-.][0-9A-Za-z.]+)?$') { throw "无效版本号：$Version" }
if (Test-Path -LiteralPath $StageRoot) { Remove-Item -LiteralPath $StageRoot -Recurse -Force }
New-Item -ItemType Directory -Force -Path $StageRoot, $ReleaseRoot | Out-Null

Push-Location $ProjectRoot
try {
    $env:YUEXU_VERSION = $Version
    cargo test --locked
    cargo build --release --locked --bin LunarCalendar
} finally {
    Remove-Item Env:YUEXU_VERSION -ErrorAction SilentlyContinue
    Pop-Location
}

Copy-Item -LiteralPath (Join-Path $ProjectRoot 'target\release\LunarCalendar.exe') -Destination (Join-Path $StageRoot 'LunarCalendar.exe') -Force

$files = @(
    @{ Source = 'installer\Install-YueXu.ps1'; Target = 'Install-YueXu.ps1' },
    @{ Source = 'installer\Uninstall-YueXu.ps1'; Target = 'Uninstall-YueXu.ps1' },
    @{ Source = 'assets\YueXu.ico'; Target = 'YueXu.ico' },
    @{ Source = 'README.md'; Target = 'README.md' },
    @{ Source = 'LICENSE'; Target = 'LICENSE' },
    @{ Source = 'CHANGELOG.md'; Target = 'CHANGELOG.md' },
    @{ Source = 'VERSION'; Target = 'VERSION' }
)
foreach ($file in $files) {
    Copy-Item -LiteralPath (Join-Path $ProjectRoot $file.Source) -Destination (Join-Path $StageRoot $file.Target) -Force
}

$binaryHash = (Get-FileHash -LiteralPath (Join-Path $StageRoot 'LunarCalendar.exe') -Algorithm SHA256).Hash
$manifest = [ordered]@{
    name = '月序 / Lunar Almanac'
    version = $Version
    platform = 'windows-x64'
    minimumWindows = 'Windows 10 x64; no browser required for wallpaper updates'
    binary = 'LunarCalendar.exe'
    renderer = 'Rust + resvg + tiny-skia'
    sha256 = $binaryHash
    generatedAt = (Get-Date).ToUniversalTime().ToString('o')
}
$manifest | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $StageRoot 'manifest.json') -Encoding UTF8

if (-not $SkipArchive) {
    if (Test-Path -LiteralPath $ArchivePath) { Remove-Item -LiteralPath $ArchivePath -Force }
    Compress-Archive -Path (Join-Path $StageRoot '*') -DestinationPath $ArchivePath -CompressionLevel Optimal
    $archiveHash = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    Set-Content -LiteralPath "$ArchivePath.sha256" -Value "$archiveHash  $(Split-Path -Leaf $ArchivePath)" -Encoding ASCII
    Write-Output "发行包：$ArchivePath"
    Write-Output "SHA256：$archiveHash"
}

Write-Output "发行目录：$StageRoot"
