$ErrorActionPreference = "SilentlyContinue"
$env:PYTHONIOENCODING = "utf-8"
$env:HF_HUB_DISABLE_SYMLINKS_WARNING = "1"
$env:MOMENTUM_MODEL_PATH = "$PSScriptRoot\momentum-agent\momentum-engine-0.5b.gguf"

# Add cargo and standard python locations if present
if ($env:USERPROFILE) {
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
    $pyDirs = @(
        "$env:USERPROFILE\AppData\Local\Programs\Python\Python311",
        "$env:USERPROFILE\AppData\Local\Programs\Python\Python311\Scripts",
        "$env:USERPROFILE\AppData\Local\Programs\Python\Python312",
        "$env:USERPROFILE\AppData\Local\Programs\Python\Python312\Scripts",
        "$env:USERPROFILE\AppData\Local\Programs\Python\Python310",
        "$env:USERPROFILE\AppData\Local\Programs\Python\Python310\Scripts"
    )
    foreach ($p in $pyDirs) {
        if (Test-Path $p) {
            $env:Path = "$p;" + $env:Path
        }
    }
}

Write-Host "Cleaning up any old processes..."
Stop-Process -Name "cargo", "node", "python", "momentum-agent", "momentum-hud" -Force -ErrorAction SilentlyContinue

# Ensure momentum-agent binary is built
$agentExe = "$PSScriptRoot\momentum-agent\target\debug\momentum-agent.exe"
if (-not (Test-Path $agentExe)) {
    Write-Host "⚙️ Building momentum-agent executable..."
    Push-Location "$PSScriptRoot\momentum-agent"
    cargo.exe build
    Pop-Location
}

Write-Host "🚀 Starting Momentum HUD (Native Desktop App)..."
Start-Process -FilePath "npx.cmd" -ArgumentList "tauri dev" -WorkingDirectory "$PSScriptRoot\momentum-hud"

Write-Host "All core systems launched!"
