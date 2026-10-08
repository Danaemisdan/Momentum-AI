import re
with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('role=msg.role', 'role=if msg.role == "agent" { "assistant" } else { &msg.role }')

with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)
