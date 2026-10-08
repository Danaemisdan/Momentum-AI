import re
with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('LlamaSampler::penalties(64, 1.1, 0.0, 0.0)', 'LlamaSampler::penalties(1024, 1.2, 0.0, 0.0)')

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
