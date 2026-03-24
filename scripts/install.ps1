param(
    [string]$InstallDir = "$HOME\\.local\\bin"
)

$ErrorActionPreference = 'Stop'
$BinName = 'devfetch'
$Repo = if ($env:DEVFETCH_REPO) { $env:DEVFETCH_REPO } else { 'harkerhand/devfetch' }
$Version = if ($env:DEVFETCH_VERSION) { $env:DEVFETCH_VERSION } else { 'latest' }
$Asset = "$BinName-windows.exe"

if ($Version -eq 'latest') {
    $DownloadUrl = "https://github.com/$Repo/releases/latest/download/$Asset"
}
else {
    $DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$Asset"
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$TmpFile = Join-Path ([System.IO.Path]::GetTempPath()) "$Asset"

Write-Host "[install] 下载 $DownloadUrl"
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TmpFile

Copy-Item -Force $TmpFile (Join-Path $InstallDir "$BinName.exe")
Remove-Item -Force $TmpFile
Write-Host "[install] 安装完成: $(Join-Path $InstallDir "$BinName.exe")"

$pathParts = ($env:Path -split ';' | ForEach-Object { $_.TrimEnd('\\') })
if ($pathParts -notcontains $InstallDir.TrimEnd('\\')) {
    Write-Host "[install] 提示: $InstallDir 不在 PATH 中，请手动加入。"
}
