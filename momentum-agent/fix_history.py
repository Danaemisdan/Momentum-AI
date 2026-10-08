import re
with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace <|start_header_id|>{role}<|end_header_id|>\n{text}<|eot_id|>\n
content = content.replace('<|start_header_id|>{role}<|end_header_id|>\\n{text}<|eot_id|>\\n', '<|im_start|>{role}\\n{text}<|im_end|>\\n')

with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)
