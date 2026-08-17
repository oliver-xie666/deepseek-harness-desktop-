# DeepSeek Harness Desktop - Windows Package Script
Write-Host "🔨 Building DeepSeek Harness Desktop (Release)..." -ForegroundColor Cyan

cargo build --release -p dsh-ui
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

$DistDir = "target\dist\deepseek-harness-desktop"
if (Test-Path $DistDir) {
    Remove-Item -Recurse -Force $DistDir
}
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

Write-Host "📦 Packaging binary and configuration..." -ForegroundColor Yellow
Copy-Item "target\release\dsh-desktop.exe" $DistDir
Copy-Item "README.md" $DistDir
Copy-Item "DESIGN.md" $DistDir

$ZipPath = "target\dist\DeepSeek-Harness-Desktop-Windows-x64.zip"
if (Test-Path $ZipPath) {
    Remove-Item -Force $ZipPath
}

Compress-Archive -Path "$DistDir\*" -DestinationPath $ZipPath

Write-Host "✅ Successfully generated package at: $ZipPath" -ForegroundColor Green
