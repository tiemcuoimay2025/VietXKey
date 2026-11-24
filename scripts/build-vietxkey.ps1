# Build the branded x64 portable application using the upstream native toolchain.
param([switch]$SkipTests)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
& (Join-Path $PSScriptRoot 'build.ps1') -Arch x64 -SkipTests:$SkipTests
if (-not $? -or $LASTEXITCODE -ne 0) { throw 'VietXKey native build failed.' }
$taskOutput = Join-Path $taskRoot 'dist\VietXKey-x64'
New-Item -ItemType Directory -Path $taskOutput -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $taskRoot 'dist\VietFlow.exe') -Destination (Join-Path $taskOutput 'VietXKey.exe') -Force
Copy-Item -LiteralPath (Join-Path $taskRoot 'dist\vietxkey_tsf.dll') -Destination (Join-Path $taskOutput 'vietxkey_tsf.dll') -Force
# A previous local build may have left the former DLL name in this output.
$legacyDll = Join-Path $taskOutput 'vietflow_tsf.dll'
if (Test-Path -LiteralPath $legacyDll -PathType Leaf) {
    Remove-Item -LiteralPath $legacyDll -Force
}
Write-Host "VietXKey portable build: $taskOutput" -ForegroundColor Green
