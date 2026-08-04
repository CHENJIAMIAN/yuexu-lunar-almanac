[CmdletBinding()]
param(
    [int]$Year = (Get-Date).Year,
    [ValidateSet('dark', 'light', 'custom')]
    [string]$Theme
)

$ProjectRoot = Split-Path -Parent $PSScriptRoot
$Binary = Join-Path $ProjectRoot 'target\debug\LunarCalendar.exe'
if (-not (Test-Path -LiteralPath $Binary)) {
    Push-Location $ProjectRoot
    try { cargo build --bin LunarCalendar } finally { Pop-Location }
}
$arguments = @('--preview', "--year=$Year")
if ($Theme) { $arguments += "--theme=$Theme" }
& $Binary @arguments
if ($LASTEXITCODE -ne 0) { throw "预览程序退出码：$LASTEXITCODE" }
