import re
with open('start_all.ps1', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('Stop-Process -Name "cargo", "node", "python", "momentum-agent" -Force', 
                          'Stop-Process -Name "cargo", "node", "python", "momentum-agent", "momentum-hud" -Force')

with open('start_all.ps1', 'w', encoding='utf-8') as f:
    f.write(content)
