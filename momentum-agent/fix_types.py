import re

with open('src/types.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Add missing fields to UIElement
content = content.replace(
    'pub aria_label: Option<String>,',
    'pub aria_label: Option<String>,\n    pub semantic_intent: Option<crate::types::Intent>,\n    pub spatial_zone: Option<String>,'
)

with open('src/types.rs', 'w', encoding='utf-8') as f:
    f.write(content)
