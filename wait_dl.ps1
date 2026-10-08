
while ((Get-Item momentum-agent\momentum-engine-0.5b.gguf).length -lt 350000000) {
    Start-Sleep -Seconds 5
}
[System.Console]::Beep(1000, 500)
Write-Host 'Model Download Complete!'

