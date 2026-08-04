[CmdletBinding()]
param(
    [int]$Year = (Get-Date).Year,
    [ValidateSet('dark', 'light')]
    [string]$Theme = 'dark'
)

$ProjectRoot = Split-Path -Parent $PSScriptRoot
$Binary = Join-Path $ProjectRoot 'target\debug\LunarCalendar.exe'
if (-not (Test-Path -LiteralPath $Binary)) {
    Push-Location $ProjectRoot
    try { cargo build --bin LunarCalendar } finally { Pop-Location }
}
& $Binary '--preview' "--year=$Year" "--theme=$Theme"
if ($LASTEXITCODE -ne 0) { throw "预览程序退出码：$LASTEXITCODE" }
