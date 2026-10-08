import re

with open('Cargo.toml', 'r', encoding='utf-8') as f:
    content = f.read()

# Remove llama-cpp-2 and encoding_rs
content = re.sub(r'llama-cpp-2\s*=\s*".*?"\n', '', content)
content = re.sub(r'encoding_rs\s*=\s*".*?"\n', '', content)

with open('Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(content)
