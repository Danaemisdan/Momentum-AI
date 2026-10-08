import re
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Upgrade STT model from base.en to small.en for much better accuracy
content = content.replace('WhisperModel("base.en"', 'WhisperModel("small.en"')

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
