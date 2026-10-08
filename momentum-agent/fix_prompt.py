import re
with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix the ChatML formatting in decide_chat
broken_prompt = '''{}<|im_start|>user

{}<|im_end|>
<|im_start|>assistant

"'''

fixed_prompt = '''<|im_end|>
{}<|im_start|>user

{}<|im_end|>
<|im_start|>assistant

"'''

content = content.replace(broken_prompt, fixed_prompt)

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
