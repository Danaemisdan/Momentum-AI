import re
with open('src/brain.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace the block that uses elements
native_block = r'''if false \{
                let window_title = elements\.iter\(\)
                    \.find\(\|e\| e\.tag\.as_deref\(\) == Some\("window"\)\)
                    \.and_then\(\|e\| e\.title\.clone\(\)\)
                    \.unwrap_or_else\(\|\| "OS Window"\.into\(\)\);
                ui_context\.push_str\(&format\("\[ACTIVE WINDOW: {}\]\n\[", window_title\)\);
                
                let buttons: Vec<String> = elements\.iter\(\)
                    \.filter\(\|e\| e\.is_clickable && !e\.text\.trim\(\)\.is_empty\(\)\)
                    \.take\(40\)
                    \.map\(\|e\| format\("'{}'", e\.text\.trim\(\)\)\)
                    \.collect\(\);
                    
                if !buttons\.is_empty\(\) \{
                    ui_context\.push_str\(&format\("Visible elements on screen they can interact with: {}\]", buttons\.join\(", "\)\)\);
                \} else \{
                    ui_context\.push_str\("\]"\);
                \}
        \}'''
content = re.sub(native_block, '', content)

with open('src/brain.rs', 'w', encoding='utf-8') as f:
    f.write(content)
