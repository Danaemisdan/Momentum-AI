$env:Path = "C:\Users\usha\AppData\Local\Programs\Python\Python311;C:\Users\usha\AppData\Local\Programs\Python\Python311\Scripts;" + $env:Path
$ErrorActionPreference = "Continue"

Write-Host "2. Setting up Momentum Audio..."
Set-Location "C:\Users\usha\Downloads\Momentum AI\momentum-audio"
python -m venv venv
.\venv\Scripts\Activate.ps1
pip install fastapi "uvicorn[standard]" websockets numpy soundfile pedalboard piper-tts rvc-python torch torchaudio

Write-Host "3. Setting up Momentum Vision..."
Set-Location "C:\Users\usha\Downloads\Momentum AI\momentum-vision"
python -m venv venv
.\venv\Scripts\Activate.ps1
pip install fastapi "uvicorn[standard]" pydantic llama_cpp_python numpy Pillow

Write-Host "All Python backends configured!"
