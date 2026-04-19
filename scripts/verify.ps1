# scripts/verify.ps1 — Run all CI checks locally before pushing.
# Usage: pwsh scripts/verify.ps1
#
# SPDX-License-Identifier: AGPL-3.0-or-later

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

<<<<<<< HEAD
$root = Split-Path -Parent $PSScriptRoot
=======
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
>>>>>>> 4cbedcb30c7f833c45ca9f75a976ca2dda72cb54
if (-not $root) { $root = (Get-Location).Path }
$tauriDir = Join-Path $root 'src-tauri'

$failed = @()

<<<<<<< HEAD
function Invoke-Step {
=======
function Run-Step {
>>>>>>> 4cbedcb30c7f833c45ca9f75a976ca2dda72cb54
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
<<<<<<< HEAD
Invoke-Step 'Frontend: format check' { npm run format:check }
Invoke-Step 'Frontend: lint' { npm run lint }
Invoke-Step 'Frontend: typecheck' { npm run typecheck }

# ---- Rust ----
Push-Location $tauriDir
Invoke-Step 'Rust: cargo fmt --check' { cargo fmt --all -- --check }
Invoke-Step 'Rust: cargo clippy' { cargo clippy --all-targets --all-features -- -D warnings }
Invoke-Step 'Rust: cargo test' { cargo test --all-features --no-fail-fast }
=======
Run-Step 'Frontend: format check' { npm run format:check }
Run-Step 'Frontend: lint' { npm run lint }
Run-Step 'Frontend: typecheck' { npm run typecheck }

# ---- Rust ----
Push-Location $tauriDir
Run-Step 'Rust: cargo fmt --check' { cargo fmt --all -- --check }
Run-Step 'Rust: cargo clippy' { cargo clippy --all-targets --all-features -- -D warnings }
Run-Step 'Rust: cargo test' { cargo test --all-features --no-fail-fast }
>>>>>>> 4cbedcb30c7f833c45ca9f75a976ca2dda72cb54
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
