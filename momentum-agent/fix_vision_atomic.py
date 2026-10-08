import re
with open('src/vision_stream.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix atomic writing to prevent PIL.UnidentifiedImageError
old_save = '''                        let temp_file = std::env::temp_dir().join("momentum_vision_current.jpg");
                        // Save directly as JPEG to temp dir
                        let _ = image.save_with_format(temp_file, ImageFormat::Jpeg);'''
new_save = '''                        let temp_dir = std::env::temp_dir();
                        let temp_file = temp_dir.join("momentum_vision_writing.jpg");
                        let final_file = temp_dir.join("momentum_vision_current.jpg");
                        // Save to temporary file first, then atomically rename to prevent read-while-write corruption
                        if image.save_with_format(&temp_file, ImageFormat::Jpeg).is_ok() {
                            let _ = std::fs::rename(temp_file, final_file);
                        }'''
content = content.replace(old_save, new_save)

with open('src/vision_stream.rs', 'w', encoding='utf-8') as f:
    f.write(content)
