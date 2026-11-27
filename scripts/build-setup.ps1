$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

Write-Host "[1/4] Running Rust security & crypto test suite..."
Push-Location "$Root\src-tauri"
cargo test
if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }
Pop-Location

Write-Host "[2/4] Building Repose release binary with embedded offline UI..."
pnpm run tauri build --no-bundle
if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }

$ReleaseDir = "$Root\dist-release"
New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null

$StageDir = "$ReleaseDir\stage"
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $StageDir
New-Item -ItemType Directory -Force -Path $StageDir | Out-Null

$ExePath = "$Root\src-tauri\target\release\repose.exe"
if (-not (Test-Path $ExePath)) { throw "Missing release binary at $ExePath" }
Copy-Item $ExePath "$StageDir\repose.exe"

Write-Host "[3/4] Packing payload.zip..."
$PayloadZip = "$ReleaseDir\payload.zip"
Remove-Item -Force -ErrorAction SilentlyContinue $PayloadZip
Compress-Archive -Path "$StageDir\*" -DestinationPath $PayloadZip -CompressionLevel Optimal

Write-Host "[4/4] Compiling Repose-Setup-1.0.0.exe via Inno Setup..."
$Iscc = "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
if (-not (Test-Path $Iscc)) {
    throw "Inno Setup 6 (ISCC.exe) not found at $Iscc"
}
& $Iscc "$Root\packaging\installer.iss"
if ($LASTEXITCODE -ne 0) { throw "ISCC.exe failed" }

$SetupExe = "$ReleaseDir\Repose-Setup-1.0.0.exe"
$Hash = (Get-FileHash -Path $SetupExe -Algorithm SHA256).Hash.ToLower()
"$Hash  Repose-Setup-1.0.0.exe" | Out-File -Encoding ascii "$ReleaseDir\SHA256SUMS.txt"

Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $StageDir
Remove-Item -Force -ErrorAction SilentlyContinue $PayloadZip

Write-Host "DONE! Setup artifact: $SetupExe"
Get-Content "$ReleaseDir\SHA256SUMS.txt"
