import re
with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('let mut sampler = LlamaSampler::greedy();', 
'''let mut sampler = LlamaSampler::chain_simple(vec![
            LlamaSampler::penalties(64, 1.1, 0.0, 0.0),
            LlamaSampler::temp(0.6),
            LlamaSampler::dist(42),
        ]);''')

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
