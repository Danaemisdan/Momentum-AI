import re
with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix the n_batch vs batch_cap assertion crash
content = content.replace('.with_n_batch(512)', '.with_n_batch(4096)')

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
