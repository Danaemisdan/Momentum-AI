import re
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()
content = content.replace('import threading\n    stt_thread = threading.Thread', 'stt_thread = threading.Thread')
content = 'import threading\n' + content
with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
