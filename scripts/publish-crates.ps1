# PowerShell helper to publish workspace crates in dependency order
# Usage: run from repository root in PowerShell
# Make sure you have run `cargo login <TOKEN>` or set $env:CARGO_REGISTRY_TOKEN
# Also ensure your git working tree is clean and you have bumped versions and committed changes.

$manifestRoot = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location $manifestRoot

function Check-Token {
    if (-not $env:CARGO_REGISTRY_TOKEN) {
        Write-Host "CARGO_REGISTRY_TOKEN is not set. Run: cargo login <TOKEN> or set the environment variable." -ForegroundColor Yellow
        return $false
    }
    return $true
}

function Ensure-CleanGit {
    $status = git status --porcelain
    if ($status) {
        Write-Host "Git working tree is not clean. Commit or stash changes before publishing." -ForegroundColor Red
        return $false
    }
    return $true
}

if (-not (Check-Token)) { exit 1 }
if (-not (Ensure-CleanGit)) { exit 1 }

# Publish order (dependencies first)
$crates = @(
    "crates/udonsharp-macros",
    "crates/udonsharp-core",
    "crates/wasm2usharp-enhanced",
    "crates/udonsharp-bindings",
    "crates/udonsharp-compiler",
    "crates/udonsharp-build",
    "crates/udonsharp-performance",
    "crates/udonsharp-cli",
    "crates/cargo-udonsharp"
)

foreach ($c in $crates) {
    $manifest = Join-Path $PWD $c
    Write-Host "Publishing $c..." -ForegroundColor Cyan
    $cmd = "cargo publish --manifest-path `"$manifest\Cargo.toml`""
    Write-Host $cmd
    $proc = Start-Process -FilePath cargo -ArgumentList @("publish","--manifest-path", "${manifest}\Cargo.toml") -NoNewWindow -Wait -PassThru
    if ($proc.ExitCode -ne 0) {
        Write-Host "Publishing $c failed with exit code $($proc.ExitCode). Aborting." -ForegroundColor Red
        exit $proc.ExitCode
    }
    Start-Sleep -Seconds 1
}

Write-Host "All crates published successfully (or at least the script completed)." -ForegroundColor Green
