# PowerShell script to run Mandrel tests for tool42 MCP server
# This script builds tool42 if needed, validates the test config, and runs the tests

param(
    [string]$MothPath = "..\..\3rdparty\codeprism\target\release\moth",
    [string]$TestConfig = "tool42-server.yaml",
    [switch]$BuildTool42 = $false,
    [switch]$ValidateOnly = $false
)

$ErrorActionPreference = "Stop"

Write-Host "=== Tool42 Mandrel Test Runner ===" -ForegroundColor Cyan
Write-Host ""

# Check if moth binary exists (try both with and without .exe extension)
$mothExe = $MothPath + ".exe"
if (Test-Path $mothExe) {
    $MothPath = $mothExe
} elseif (-not (Test-Path $MothPath)) {
    Write-Host "ERROR: moth binary not found at: $MothPath or $mothExe" -ForegroundColor Red
    Write-Host "Please build moth first:" -ForegroundColor Yellow
    Write-Host "  cd 3rdparty\codeprism" -ForegroundColor Yellow
    Write-Host "  cargo build --release --bin moth" -ForegroundColor Yellow
    exit 1
}

Write-Host "Using moth binary: $MothPath" -ForegroundColor Green

# Build tool42 if requested
if ($BuildTool42) {
    Write-Host "Building tool42..." -ForegroundColor Yellow
    $buildResult = cargo build --release 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Failed to build tool42" -ForegroundColor Red
        Write-Host $buildResult
        exit 1
    }
    Write-Host "tool42 built successfully" -ForegroundColor Green
}

# Check if tool42 is available
$tool42Path = Get-Command tool42 -ErrorAction SilentlyContinue
if (-not $tool42Path) {
    Write-Host "WARNING: tool42 not found in PATH" -ForegroundColor Yellow
    Write-Host "Make sure tool42 is installed or built" -ForegroundColor Yellow
}

# Validate test configuration
Write-Host ""
Write-Host "Validating test configuration..." -ForegroundColor Yellow
$validateResult = & $MothPath validate $TestConfig 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Test configuration validation failed" -ForegroundColor Red
    Write-Host $validateResult
    exit 1
}
Write-Host "Test configuration is valid" -ForegroundColor Green

if ($ValidateOnly) {
    Write-Host ""
    Write-Host "Validation complete (--ValidateOnly specified)" -ForegroundColor Green
    exit 0
}

# Run tests
Write-Host ""
Write-Host "Running Mandrel tests..." -ForegroundColor Yellow
Write-Host ""

$testResult = & $MothPath run $TestConfig 2>&1
$exitCode = $LASTEXITCODE

Write-Host ""
if ($exitCode -eq 0) {
    Write-Host "=== All tests passed! ===" -ForegroundColor Green
} else {
    Write-Host "=== Some tests failed ===" -ForegroundColor Red
    Write-Host $testResult
}

exit $exitCode

