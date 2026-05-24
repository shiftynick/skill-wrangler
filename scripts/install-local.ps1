# Build Skill Wrangler release and install to %USERPROFILE%\.local\bin
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host "Building release (npm run tauri build)..."
npm run tauri build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$BinaryName = "skill-wrangler.exe"
$ReleaseBinary = Join-Path $RepoRoot "src-tauri\target\release\$BinaryName"
if (-not (Test-Path $ReleaseBinary)) {
    Write-Error "Release binary not found at $ReleaseBinary"
    exit 1
}

$InstallDir = Join-Path $env:USERPROFILE ".local\bin"
$Dest = Join-Path $InstallDir $BinaryName

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Copy-Item -Path $ReleaseBinary -Destination $Dest -Force
Write-Host "Installed Skill Wrangler to $Dest"
