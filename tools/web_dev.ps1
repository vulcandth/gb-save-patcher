$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$webCrate = Join-Path $repoRoot 'crates\gb-save-web'
$wwwDir = Join-Path $webCrate 'www'

Write-Host "Building WASM package..." -ForegroundColor Cyan
Push-Location $webCrate

if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
  throw "wasm-pack is not installed. Install with: cargo install wasm-pack"
}

wasm-pack build --target web --out-dir www/pkg

Pop-Location

Write-Host "Serving web UI at http://localhost:8000/" -ForegroundColor Green
Write-Host "(Press Ctrl+C to stop)" -ForegroundColor DarkGray
Push-Location $wwwDir
python -m http.server 8000
Pop-Location
