import re
with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('<|begin_of_text|><|start_header_id|>system<|end_header_id|>', '<|im_start|>system')
content = content.replace('<|start_header_id|>system<|end_header_id|>', '<|im_start|>system')
content = content.replace('<|end_header_id|>', '')
content = content.replace('<|start_header_id|>', '')

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
