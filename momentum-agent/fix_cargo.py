import re

with open('Cargo.toml', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('llama-cpp-2 = "0.1.140"', 'llama-cpp-2 = "=0.1.140"')

with open('Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(content)
