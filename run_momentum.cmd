@echo off
echo ===================================================
echo Starting Momentum AI (Interactive Desktop Mode)...
echo ===================================================
set PYTHONIOENCODING=utf-8
set HF_HUB_DISABLE_SYMLINKS_WARNING=1
set PATH=%PATH%;%USERPROFILE%\.cargo\bin
echo ⚙️ Compiling momentum-agent if needed...
cd /d "%~dp0momentum-agent"
cargo.exe build
cd /d "%~dp0momentum-hud"
npx.cmd tauri dev
pause
