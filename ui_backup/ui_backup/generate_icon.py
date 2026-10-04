from PIL import Image, ImageDraw, ImageFilter

def draw_pill(draw, box, radius, fill):
    x0, y0, x1, y1 = box
    draw.rectangle([x0, y0 + radius, x1, y1 - radius], fill=fill)
    draw.rectangle([x0 + radius, y0, x1 - radius, y1], fill=fill)
    draw.pieslice([x0, y0, x0 + radius * 2, y0 + radius * 2], 180, 270, fill=fill)
    draw.pieslice([x1 - radius * 2, y0, x1, y0 + radius * 2], 270, 360, fill=fill)
    draw.pieslice([x0, y1 - radius * 2, x0 + radius * 2, y1], 90, 180, fill=fill)
    draw.pieslice([x1 - radius * 2, y1 - radius * 2, x1, y1], 0, 90, fill=fill)

size = 1024
img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
glow_layer = Image.new('RGBA', (size, size), (0, 0, 0, 0))

draw = ImageDraw.Draw(img)
glow_draw = ImageDraw.Draw(glow_layer)

pill_w = 140
pill_h = 360
radius = 70
spacing = 160
center_x = size // 2
center_y = size // 2

# Draw glow
glow_color = (139, 92, 246, 255) # Purple glow
draw_pill(glow_draw, [center_x - spacing//2 - pill_w, center_y - pill_h//2, center_x - spacing//2, center_y + pill_h//2], radius, glow_color)
draw_pill(glow_draw, [center_x + spacing//2, center_y - pill_h//2, center_x + spacing//2 + pill_w, center_y + pill_h//2], radius, glow_color)

glow_layer = glow_layer.filter(ImageFilter.GaussianBlur(60))
img.alpha_composite(glow_layer)

# Draw solid pills
draw_pill(draw, [center_x - spacing//2 - pill_w, center_y - pill_h//2, center_x - spacing//2, center_y + pill_h//2], radius, (255, 255, 255, 255))
draw_pill(draw, [center_x + spacing//2, center_y - pill_h//2, center_x + spacing//2 + pill_w, center_y + pill_h//2], radius, (255, 255, 255, 255))

img.save('src-tauri/icons/icon.png', 'PNG')
img.resize((128, 128), Image.LANCZOS).save('src-tauri/icons/128x128.png', 'PNG')
img.resize((32, 32), Image.LANCZOS).save('src-tauri/icons/32x32.png', 'PNG')

print("Icons generated!")
