import re

with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Remove model_path logic
content = re.sub(r'let model_path = .*?\n.*?expect\("Could not find GGUF model.*?;\n', '', content, flags=re.DOTALL)
content = re.sub(r'let brain = Arc::new\(MomentumBrain::init\(&model_path\)\?\);', 'let brain = Arc::new(MomentumBrain::new());', content)
content = re.sub(r'let backend = llama_cpp_2::llama_backend::LlamaBackend::init\(\)\.expect\("Failed to init llama backend"\);\n', '', content)

with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)
