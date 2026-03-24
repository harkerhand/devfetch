param(
    [string]$InstallDir = "$HOME\\.local\\bin"
)

$ErrorActionPreference = 'Stop'
$BinName = 'devfetch.exe'
$Target = Join-Path $InstallDir $BinName

if (Test-Path $Target) {
    Remove-Item -Force $Target
    Write-Host "[uninstall] 已删除 $Target"
}
else {
    Write-Host "[uninstall] 未找到 $Target"
}
