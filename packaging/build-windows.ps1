# Build and package n0ne for Windows
# Usage: .\packaging\build-windows.ps1 [version]
# Output: n0ne-windows-x86_64.zip

param([string]$Version = "dev")

$Target  = "x86_64-pc-windows-msvc"
$Dist    = "dist\n0ne"
$Archive = "n0ne-windows-x86_64.zip"

Write-Host "==> Building n0ne $Version for Windows..."
cargo build --release --target $Target --bin n0ne

Write-Host "==> Packaging..."
Remove-Item -Recurse -Force dist -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

Copy-Item "target\$Target\release\n0ne.exe" $Dist\
Copy-Item README.md                          $Dist\
Copy-Item LICENSE                            $Dist\
Copy-Item -Recurse examples                  $Dist\

Remove-Item $Archive -ErrorAction SilentlyContinue
Compress-Archive -Path $Dist -DestinationPath $Archive

$size = (Get-Item $Archive).Length / 1MB
Write-Host ""
Write-Host "  Done: $Archive"
Write-Host "  Size: $([math]::Round($size, 1)) MB"
