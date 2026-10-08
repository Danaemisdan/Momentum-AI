import re

with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = re.sub(r'// LlamaContext is not Send by default.*?\n.*?pub struct SendContext.*?unsafe impl Send for SendContext \{\}\n', '', content, flags=re.DOTALL)

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
