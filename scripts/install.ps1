# n0ne installer for Windows — https://github.com/loyality7/n0ne
# Usage: irm https://raw.githubusercontent.com/loyality7/n0ne/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo     = "loyality7/n0ne"
$BinName  = "n0ne.exe"
$InstDir  = "$env:LOCALAPPDATA\n0ne"

# ── Check / install clang ─────────────────────────────────────────────────────
function Test-Clang {
    $found = Get-Command clang -ErrorAction SilentlyContinue
    return $null -ne $found
}

function Install-Clang {
    Write-Host "==> clang not found — installing via winget..." -ForegroundColor Cyan

    # Try winget first (built-in on Windows 11 / updated Windows 10)
    $winget = Get-Command winget -ErrorAction SilentlyContinue
    if ($winget) {
        winget install --id LLVM.LLVM -e --accept-package-agreements --accept-source-agreements
        # Refresh PATH for current session
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" +
                    [System.Environment]::GetEnvironmentVariable("PATH", "User")
        return
    }

    # Fall back to choco
    $choco = Get-Command choco -ErrorAction SilentlyContinue
    if ($choco) {
        choco install llvm -y
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" +
                    [System.Environment]::GetEnvironmentVariable("PATH", "User")
        return
    }

    Write-Host ""
    Write-Host "error: could not install clang automatically." -ForegroundColor Red
    Write-Host "  Please install LLVM manually from: https://releases.llvm.org/download.html"
    Write-Host "  Then re-run this script."
    exit 1
}

if (-not (Test-Clang)) {
    Install-Clang
}

$clangCmd = Get-Command clang -ErrorAction SilentlyContinue
if (-not $clangCmd) {
    # Check default LLVM installation path in Windows
    if (Test-Path "C:\Program Files\LLVM\bin\clang.exe") {
        $env:PATH += ";C:\Program Files\LLVM\bin"
        $clangCmd = Get-Command clang -ErrorAction SilentlyContinue
    }
}

$clangPath = if ($clangCmd) { $clangCmd.Source } else { "Clang installed (please restart terminal to refresh PATH)" }
Write-Host "==> clang found: $clangPath"

# ── Fetch latest release ──────────────────────────────────────────────────────
Write-Host "==> Fetching latest n0ne release..."
$apiUrl  = "https://api.github.com/repos/$Repo/releases/latest"
$release = Invoke-RestMethod -Uri $apiUrl -Headers @{ "User-Agent" = "n0ne-installer" }
$tag     = $release.tag_name

if (-not $tag) {
    Write-Host "error: could not fetch latest release from GitHub." -ForegroundColor Red
    exit 1
}

Write-Host "==> Latest release: $tag"

# ── Download binary ───────────────────────────────────────────────────────────
$artifact = "n0ne-windows-x86_64.zip"
$url      = "https://github.com/$Repo/releases/download/$tag/$artifact"
$tmp      = Join-Path $env:TEMP "n0ne-install"

New-Item -ItemType Directory -Force -Path $tmp | Out-Null
$zipPath = Join-Path $tmp $artifact

Write-Host "==> Downloading $artifact..."
Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

Write-Host "==> Extracting..."
Expand-Archive -Path $zipPath -DestinationPath $tmp -Force

# ── Install binary ────────────────────────────────────────────────────────────
$binary = Join-Path $tmp "n0ne\$BinName"
if (-not (Test-Path $binary)) {
    Write-Host "error: binary not found in archive." -ForegroundColor Red
    exit 1
}

New-Item -ItemType Directory -Force -Path $InstDir | Out-Null
$targetPath = Join-Path $InstDir $BinName
if (Test-Path $targetPath) {
    $oldPath = "$targetPath.old"
    if (Test-Path $oldPath) {
        Remove-Item $oldPath -Force -ErrorAction SilentlyContinue
    }
    Rename-Item $targetPath ($BinName + ".old") -Force -ErrorAction SilentlyContinue
}
Copy-Item $binary $targetPath -Force
Remove-Item $tmp -Recurse -Force

# ── Add to user PATH if not already there ────────────────────────────────────
$userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$InstDir*") {
    [System.Environment]::SetEnvironmentVariable(
        "PATH", "$userPath;$InstDir", "User"
    )
    Write-Host "==> Added $InstDir to your PATH."
    Write-Host "    Restart your terminal for it to take effect."
}

# ── Done ──────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  n0ne $tag installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "  Get started:"
Write-Host "    n0ne run hello.n0"
Write-Host ""
