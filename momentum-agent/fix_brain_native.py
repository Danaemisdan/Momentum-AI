import re
with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = content.replace('if let Ok(elements) = crate::perception::perceive_native().await {', 'if false {')
with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
