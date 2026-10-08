import re
with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Remove the system note injection
content = content.replace('msg_with_vision = format!("[System Note: The user\'s currently focused application is \'{}\']\\n{}", active_app, msg_with_vision);', '// Removed system note injection for Windows/0.5B compatibility')

with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)
