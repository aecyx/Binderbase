# scripts/verify.ps1 — Run all CI checks locally before pushing.
# Usage: pwsh scripts/verify.ps1
#
# SPDX-License-Identifier: AGPL-3.0-or-later

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
if (-not $root) { $root = (Get-Location).Path }
$tauriDir = Join-Path $root 'src-tauri'

$failed = @()

function Invoke-Step {
    param([string]$Label, [scriptblock]$Action)
    Write-Host "`n=== $Label ===" -ForegroundColor Cyan
    try {
        & $Action
        if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) { throw "exit code $LASTEXITCODE" }
        Write-Host "  PASS" -ForegroundColor Green
    } catch {
        Write-Host "  FAIL: $_" -ForegroundColor Red
        $script:failed += $Label
    }
}

Push-Location $root

# ---- Frontend ----
Invoke-Step 'Frontend: format check' { npm run format:check }
Invoke-Step 'Frontend: lint' { npm run lint }
Invoke-Step 'Frontend: typecheck' { npm run typecheck }

# ---- Rust ----
Push-Location $tauriDir
Invoke-Step 'Rust: cargo fmt --check' { cargo fmt --all -- --check }
Invoke-Step 'Rust: cargo clippy' { cargo clippy --all-targets --all-features -- -D warnings }
Invoke-Step 'Rust: cargo test' { cargo test --all-features --no-fail-fast }
Pop-Location

Pop-Location

# ---- Summary ----
Write-Host "`n========================================" -ForegroundColor Cyan
if ($failed.Count -eq 0) {
    Write-Host "All checks passed." -ForegroundColor Green
    exit 0
} else {
    Write-Host "FAILED steps:" -ForegroundColor Red
    $failed | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}
