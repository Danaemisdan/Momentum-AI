import re
with open('engine.py', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('''    if llm is None:
        raise HTTPException(status_code=500, detail="Vision model not loaded.")''', '')

with open('engine.py', 'w', encoding='utf-8') as f:
    f.write(content)
