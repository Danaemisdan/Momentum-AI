import re
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

content = re.sub(r'print\((.*?)\)', r'print(\1, flush=True)', content)
# Fix double flush=True
content = content.replace(', flush=True, flush=True)', ', flush=True)')

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
