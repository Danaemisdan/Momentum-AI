import re
with open('src/vision_stream.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('use image::ImageFormat;', 'use xcap::image::ImageFormat;')

with open('src/vision_stream.rs', 'w', encoding='utf-8') as f:
    f.write(content)
