$ErrorActionPreference = "Continue"

Write-Host "1. Setting up Momentum Audio..."
Set-Location "$PSScriptRoot\momentum-audio"
python -m venv venv
.\venv\Scripts\Activate.ps1
pip install fastapi "uvicorn[standard]" websockets numpy soundfile pedalboard piper-tts rvc-python torch torchaudio

Write-Host "2. Setting up Momentum Vision..."
Set-Location "$PSScriptRoot\momentum-vision"
python -m venv venv
.\venv\Scripts\Activate.ps1
pip install fastapi "uvicorn[standard]" pydantic llama_cpp_python numpy Pillow

Write-Host "All Python backends configured!"
