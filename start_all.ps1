$env:Path = "C:\Users\usha\AppData\Local\Programs\Python\Python311;C:\Users\usha\AppData\Local\Programs\Python\Python311\Scripts;" + $env:Path
$env:PYTHONIOENCODING = "utf-8"
$env:MOMENTUM_MODEL_PATH = "C:\Users\usha\Downloads\Momentum AI\momentum-agent\momentum-engine-0.5b.gguf"
$ErrorActionPreference = "SilentlyContinue"

Write-Host "Cleaning up any old processes..."
Stop-Process -Name "cargo", "node", "python", "momentum-agent", "momentum-hud" -Force

Write-Host "Starting Momentum HUD (Native Desktop App)..."
$env:Path += ";$HOME\.cargo\bin"
Start-Process -FilePath "npx.cmd" -ArgumentList "tauri dev" -WorkingDirectory ".\momentum-hud"

Write-Host "All core systems have been launched in the background!"
