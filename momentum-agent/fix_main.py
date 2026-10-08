import re

with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Remove backend init and modify brain init
content = re.sub(r'let backend = llama_cpp_2::llama_backend::LlamaBackend::init\(\)\.expect\("Failed to init llama backend"\);\n', '', content)
content = re.sub(r'let brain = Arc::new\(MomentumBrain::new\(backend, &model_path\)\.expect\("Failed to load model"\)\);', 'let brain = Arc::new(MomentumBrain::new());', content)
content = re.sub(r'let brain = Arc::new\(MomentumBrain::new\(&model_path\)\.expect\("Failed to load model"\)\);', 'let brain = Arc::new(MomentumBrain::new());', content)

with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)
