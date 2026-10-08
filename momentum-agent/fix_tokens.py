import re

with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace Llama 3 tokens with ChatML tokens
content = content.replace('<|begin_of_text|><|start_header_id|>system<|end_header_id|>', '<|im_start|>system\n')
content = content.replace('<|eot_id|><|start_header_id|>user<|end_header_id|>', '<|im_end|>\n<|im_start|>user\n')
content = content.replace('<|eot_id|><|start_header_id|>assistant<|end_header_id|>', '<|im_end|>\n<|im_start|>assistant\n')

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
