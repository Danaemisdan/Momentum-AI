@echo off
echo ===================================================
echo Starting Momentum AI (Interactive Desktop Mode)...
echo ===================================================
set PYTHONIOENCODING=utf-8
set HF_HUB_DISABLE_SYMLINKS_WARNING=1
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
set MOMENTUM_MODEL_PATH=C:\Users\usha\Downloads\Momentum AI\momentum-agent\momentum-engine-3b.gguf
cd /d "C:\Users\usha\Downloads\Momentum AI\momentum-hud"
npx.cmd tauri dev
pause
