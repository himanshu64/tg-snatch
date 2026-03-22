# tg-snatch installer for Windows
# Usage: irm https://raw.githubusercontent.com/himanshu64/tg-snatch/main/packaging/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "himanshu64/tg-snatch"
$Binary = "tg-snatch"
$InstallDir = "$env:LOCALAPPDATA\tg-snatch\bin"

function Write-Info($msg) { Write-Host ">>> $msg" -ForegroundColor Cyan }
function Write-Ok($msg) { Write-Host ">>> $msg" -ForegroundColor Green }
function Write-Err($msg) { Write-Host "ERROR: $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "  ╔════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "  ║  tg-snatch  ·  Windows Installer       ║" -ForegroundColor Cyan
Write-Host "  ╚════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Get latest version
Write-Info "Fetching latest version..."
try {
    $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $Release.tag_name
} catch {
    Write-Err "Could not fetch latest version. Check https://github.com/$Repo/releases"
}
Write-Info "Latest version: $Version"

# Download
$Url = "https://github.com/$Repo/releases/download/$Version/$Binary-x86_64-pc-windows-msvc.zip"
$TmpDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
$ZipPath = Join-Path $TmpDir "$Binary.zip"

Write-Info "Downloading $Binary $Version..."
try {
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
} catch {
    Write-Err "Download failed. Check if release exists: $Url"
}

# Extract
Write-Info "Extracting..."
Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

# Install
Write-Info "Installing to $InstallDir..."
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Copy-Item (Join-Path $TmpDir "$Binary.exe") (Join-Path $InstallDir "$Binary.exe") -Force

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Info "Adding to PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
}

# Cleanup
Remove-Item -Recurse -Force $TmpDir

Write-Ok "tg-snatch $Version installed successfully!"
Write-Host ""
Write-Host "  Restart your terminal, then run 'tg-snatch' to get started."
Write-Host ""
