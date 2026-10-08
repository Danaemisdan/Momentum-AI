import sys
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

content = 'import sys\nsys.stdout.reconfigure(encoding="utf-8")\nsys.stderr.reconfigure(encoding="utf-8")\n' + content

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
