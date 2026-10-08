import re
with open('src/brain.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

new_lines = []
skip = False
for line in lines:
    if 'if false {' in line:
        skip = True
        continue
    if skip and '}' in line and 'ui_context.push_str("]");' not in line and '}' not in new_lines[-1] if new_lines else True:
        # Wait, if skip is true, we just skip until we find the closing brace.
        # It's better to just delete lines 746-760. Let's just catch the error lines.
        pass

# Actually let's just comment out `elements.iter()` and replace with `[].iter()`
with open('src/brain.rs', 'w', encoding='utf-8') as f:
    for line in lines:
        if 'elements.iter()' in line:
            f.write(line.replace('elements.iter()', 'Vec::<crate::types::UIElement>::new().iter()'))
        else:
            f.write(line)
