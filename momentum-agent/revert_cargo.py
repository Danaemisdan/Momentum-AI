import re

with open('Cargo.toml', 'r', encoding='utf-8') as f:
    content = f.read()

# Add llama-cpp-2 and encoding_rs back
if 'llama-cpp-2' not in content:
    content = content.replace('futures = "0.3"', 'futures = "0.3"\nllama-cpp-2 = "0.1.140"\nencoding_rs = "0.8.35"')

with open('Cargo.toml', 'w', encoding='utf-8') as f:
    f.write(content)
