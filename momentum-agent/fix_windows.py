import re

with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix active_app
active_app_block = r'''                            let active_app = std::process::Command::new\("osascript"\)
                                \.arg\("-e"\)
                                \.arg\("tell application \\"System Events\\" to get name of first application process whose frontmost is true"\)
                                \.output\(\)
                                \.map\(\|o\| String::from_utf8_lossy\(&o\.stdout\)\.trim\(\)\.to_string\(\)\)
                                \.unwrap_or_else\(\|_\| "Desktop"\.into\(\)\);'''

new_active_app = '''                            let active_app = "Windows Desktop".to_string();'''
content = re.sub(active_app_block, new_active_app, content)

# Fix open app
open_app_block = r'''                                            let _ = std::process::Command::new\("open"\)
                                                \.arg\("-a"\)
                                                \.arg\(app_name\)
                                                \.spawn\(\);'''

new_open_app = '''                                            let _ = std::process::Command::new("cmd")
                                                .arg("/C")
                                                .arg(format!("start {}", app_name))
                                                .spawn();'''
content = re.sub(open_app_block, new_open_app, content)

with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)
