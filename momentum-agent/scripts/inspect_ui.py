#!/usr/bin/env python3
"""
Momentum UI Visual Inspector & Semantic Label Debugger
Captures the screen, parses live Accessibility tree via mac_ax across visible windows,
draws bounding boxes and badges on screen, and prints a rich semantic label table.
"""

import subprocess
import json
import os
import sys
import time
from PIL import Image, ImageDraw, ImageFont

def clean_role(role: str, text: str = "", placeholder: str = "") -> str:
    r = role.lower()
    combined = (text + " " + placeholder).lower()
    
    if "combobox" in r:
        if "search" in combined:
            return "search field"
        return "input field"

    mapping = {
        "axbutton": "button",
        "button": "button",
        "axtextfield": "input field",
        "axtextarea": "input field",
        "edit": "input field",
        "input": "input field",
        "axsearchfield": "search field",
        "searchbox": "search field",
        "axlink": "link",
        "link": "link",
        "hyperlink": "link",
        "axpopupbutton": "dropdown",
        "axcheckbox": "checkbox",
        "checkbox": "checkbox",
        "axradiobutton": "radio button",
        "radiobutton": "radio button",
        "axmenuitem": "menu item",
        "menuitem": "menu item",
        "axtabitem": "tab",
        "tabitem": "tab",
        "aximage": "image",
        "image": "image",
        "axslider": "slider",
        "slider": "slider",
        "axheading": "heading",
        "heading": "heading",
        "axstatictext": "text",
        "statictext": "text",
        "axgroup": "control",
        "group": "control",
        "axdockitem": "dockitem",
        "axwindowmovehandle": "move handle",
        "axwindowresizecorner": "resize corner",
        "axwindowresizeborder": "resize border",
    }
    return mapping.get(r, r.replace("ax", ""))

def compute_semantic_label(role: str, text: str, title: str, placeholder: str, desc: str, container: str) -> str:
    r = clean_role(role, text or title, placeholder)
    parts = []
    seen = set()

    for cand in [text, title, desc]:
        if cand:
            trimmed = cand.strip()
            lower = trimmed.lower()
            if trimmed and lower not in seen:
                seen.add(lower)
                parts.append(trimmed)

    # Fallback to placeholder only if no primary label was found
    if not parts and placeholder:
        trimmed = placeholder.strip()
        lower = trimmed.lower()
        if trimmed and lower not in seen:
            seen.add(lower)
            parts.append(trimmed)

    if not parts:
        main_desc = r
    else:
        main_desc = f"{' '.join(parts)} {r}"

    if container and container != "window":
        return f"{main_desc} in {container.strip()}"
    return main_desc

def trunc(text: str, max_len: int) -> str:
    if not text:
        return ""
    text = text.strip().replace("\n", " ")
    if len(text) > max_len:
        return text[:max_len - 2] + ".."
    return text

def get_color(container: str, role: str = ""):
    c = container.lower()
    r = role.lower()
    if "dock" in c:
        return (0, 255, 200)       # Bright Teal / Mint (macOS Dock)
    elif "status bar" in c:
        return (255, 140, 0)       # Bright Amber / Orange (Top-Right System Status Bar)
    elif "menu bar" in c:
        return (255, 80, 220)      # Bright Magenta (System MenuBar)
    elif "move handle" in r or "resize" in r:
        return (255, 215, 0)       # Bright Gold (Window Spatial Affordances)
    elif "web document" in c:
        return (0, 220, 255)       # Cyan (Web / YouTube)
    elif "tabs" in c:
        return (255, 180, 0)       # Gold
    elif "toolbar" in c:
        return (255, 60, 120)      # Pink / Magenta
    elif "menu" in c:
        return (180, 100, 255)     # Purple
    elif "dialog" in c or "sheet" in c:
        return (255, 50, 50)       # Red
    elif "workspace" in c:
        return (0, 255, 120)       # Lime Green
    elif "timeline" in c:
        return (50, 150, 255)      # Blue
    elif "sidebar" in c:
        return (255, 230, 0)       # Yellow
    elif "preview" in c:
        return (100, 200, 255)     # Sky Blue
    return (180, 180, 180)         # Gray

def main():
    target_app = sys.argv[1] if len(sys.argv) > 1 else None

    # Close previous debug screenshot in Preview so it doesn't occlude the active screen
    try:
        subprocess.run(
            ["osascript", "-e", 'tell application "Preview" to close windows'],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        )
        time.sleep(0.15)
    except Exception:
        pass
    
    # 1. Capture screen
    screen_path = "/tmp/momentum_screen_raw.png"
    subprocess.run(["screencapture", "-x", screen_path], check=True)
    img = Image.open(screen_path)
    img_w, img_h = img.size

    # 2. Run mac_ax to get UI tree
    script_dir = os.path.dirname(os.path.abspath(__file__))
    mac_ax_path = os.path.join(script_dir, "mac_ax")
    if not os.path.exists(mac_ax_path):
        mac_ax_path = os.path.join(script_dir, "../momentum-agent/scripts/mac_ax")

    cmd = [mac_ax_path]
    if target_app:
        cmd.append(target_app)
    
    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        nodes = json.loads(res.stdout)
    except Exception as e:
        print(f"Error parsing mac_ax json: {e}\nRaw output: {res.stdout[:200]}")
        sys.exit(1)

    if not nodes:
        print("No UI elements found for target.")
        sys.exit(0)

    # Calculate retina display scaling factor
    max_ax_x = max(n.get("x", 0) + (n.get("w", 0) / 2) for n in nodes)
    scale = 2.0 if img_w > 2000 and max_ax_x <= 2000 else 1.0

    draw = ImageDraw.Draw(img)
    
    try:
        font = ImageFont.truetype("/System/Library/Fonts/SFNSMono.ttf", int(13 * scale))
        badge_font = ImageFont.truetype("/System/Library/Fonts/HelveticaNeue.ttc", int(11 * scale))
    except Exception:
        font = ImageFont.load_default()
        badge_font = font

    print("\n" + "="*175)
    print(f"{'#':<4} | {'APP':<14} | {'ROLE':<13} | {'CONTAINER':<18} | {'LABEL':<22} | {'PURPOSE':<28} | {'DESTINATION':<24} | {'HUMAN INTENT'}")
    print("="*175)

    interactive_roles = {
        "button", "link", "input field", "search field", "dropdown", "radio button",
        "checkbox", "slider", "menu item", "tab", "dockitem", "menubaritem",
        "move handle", "resize corner", "resize border"
    }

    occupied_badge_rects = []
    annotated_count = 0

    for idx, node in enumerate(nodes, start=1):
        x = node.get("x", 0)
        y = node.get("y", 0)
        w = node.get("w", 0)
        h = node.get("h", 0)
        app = node.get("app", "")
        role = node.get("role", "")
        title = node.get("title", "")
        desc = node.get("description", "")
        placeholder = node.get("placeholder", "")
        val = node.get("value", "")
        container = node.get("container", "window")
        action = node.get("action", "click")
        purpose = node.get("purpose", "")
        destination = node.get("destination", "")
        human_intent = node.get("human_intent", "")

        semantic_label = compute_semantic_label(role, title or val, title, placeholder, desc, container)

        app_str = trunc(app, 14)
        clean_r = clean_role(role, title or val, placeholder)
        role_str = trunc(clean_r, 13)
        container_str = trunc(container, 18)
        label_str = trunc(semantic_label, 22)
        purpose_str = trunc(purpose, 28)
        dest_str = trunc(destination, 24)
        intent_str = trunc(human_intent, 40)

        print(f"[{idx:>2}] | {app_str:<14} | {role_str:<13} | {container_str:<18} | {label_str:<22} | {purpose_str:<28} | {dest_str:<24} | {intent_str}")

        left = (x - (w / 2)) * scale
        top = (y - (h / 2)) * scale
        right = (x + (w / 2)) * scale
        bottom = (y + (h / 2)) * scale

        if w * scale >= img_w * 0.95 and h * scale >= img_h * 0.95:
            continue

        box_color = get_color(container, clean_r)

        draw.rectangle([left, top, right, bottom], outline=box_color, width=max(2, int(2 * scale)))

        is_interactive = clean_r in interactive_roles or "dock" in role.lower() or "menu" in role.lower()
        is_dock = "dock" in container.lower() or "dock" in role.lower()
        is_status_bar = "status bar" in container.lower()
        
        if is_interactive:
            short_label = (title or desc or val or "").strip()
            if not short_label and purpose:
                short_label = purpose.strip()
            
            if is_dock:
                # Keep dock badges clean and compact so they never crowd each other
                if len(short_label) > 13:
                    short_label = short_label[:11] + ".."
                badge_text = f"[{idx}] {short_label}" if short_label else f"[{idx}] dock"
                badge_w = len(badge_text) * int(7 * scale)
                badge_h = int(16 * scale)
                # Two alternating tiers directly above the dock icons to prevent horizontal collisions
                if idx % 2 == 0:
                    badge_top = max(0, top - badge_h - int(4 * scale))
                else:
                    badge_top = max(0, top - (2 * badge_h) - int(8 * scale))
                badge_left = max(2, min(left, img_w - badge_w - 2))
            elif is_status_bar:
                # Top-right status items: display concise title below menu bar
                if len(short_label) > 18:
                    short_label = short_label[:16] + ".."
                badge_text = f"[{idx}] {short_label}" if short_label else f"[{idx}] status"
                badge_w = len(badge_text) * int(7 * scale)
                badge_h = int(16 * scale)
                badge_top = top + (h * scale) + int(2 * scale)
                badge_left = max(2, min(left, img_w - badge_w - 2))
                for ox1, oy1, ox2, oy2 in occupied_badge_rects:
                    if not (badge_left + badge_w < ox1 or badge_left > ox2 or badge_top + badge_h < oy1 or badge_top > oy2):
                        badge_top = max(oy2 + 2, badge_top)
            else:
                if len(short_label) > 16:
                    short_label = short_label[:14] + ".."
                
                if short_label:
                    badge_text = f"[{idx}] {clean_r}: {short_label}"
                else:
                    badge_text = f"[{idx}] {clean_r}"
                    
                badge_w = len(badge_text) * int(7 * scale)
                badge_h = int(16 * scale)
                badge_top = max(0, top - badge_h)
                badge_left = max(2, min(left, img_w - badge_w - 2))

                # Anti-collision: shift if colliding with existing badge
                for ox1, oy1, ox2, oy2 in occupied_badge_rects:
                    if not (badge_left + badge_w < ox1 or badge_left > ox2 or badge_top + badge_h < oy1 or badge_top > oy2):
                        # If shifting downwards would push badge off screen, shift upwards instead
                        if top + (h * scale) + badge_h + 4 < img_h:
                            badge_top = max(oy2 + 2, top + (h * scale) + 2)
                        else:
                            badge_top = max(0, oy1 - badge_h - 2)
                        
            # Ensure badge is strictly within visible canvas
            badge_top = max(0, min(badge_top, img_h - badge_h - 2))
            badge_left = max(2, min(badge_left, img_w - badge_w - 2))

            draw.rectangle([badge_left, badge_top, badge_left + badge_w, badge_top + badge_h], fill=box_color)
            draw.text((badge_left + 3, badge_top + 1), badge_text, fill=(0, 0, 0), font=badge_font)
            occupied_badge_rects.append((badge_left, badge_top, badge_left + badge_w, badge_top + badge_h))
            annotated_count += 1
        elif "heading" in clean_r:
            badge_text = f"[{idx}] H"
            badge_w = len(badge_text) * int(7 * scale)
            badge_h = int(14 * scale)
            badge_top = max(0, min(top - badge_h, img_h - badge_h - 2))
            badge_left = max(2, min(left, img_w - badge_w - 2))
            draw.rectangle([badge_left, badge_top, badge_left + badge_w, badge_top + badge_h], fill=box_color)
            draw.text((badge_left + 3, badge_top + 1), badge_text, fill=(0, 0, 0), font=badge_font)
            occupied_badge_rects.append((badge_left, badge_top, badge_left + badge_w, badge_top + badge_h))
            annotated_count += 1

    print("="*175)
    print(f"\n📊 Total elements detected: {len(nodes)} (Annotated {annotated_count} UI boxes)")
    
    annotated_path = "/tmp/momentum_ui_debug.png"
    img.save(annotated_path)
    print(f"🖼️ Annotated screenshot saved to: {annotated_path}")

    # Also save to conversation artifacts directory if available
    artifact_dir = "/Users/sanjeevn/.gemini/antigravity/brain/56be5417-a621-4c45-b7b8-f74c1d6a395a"
    if os.path.exists(artifact_dir):
        for fname in ["momentum_ui_debug_v4.png", "momentum_ui_debug_v3.png"]:
            p = os.path.join(artifact_dir, fname)
            img.save(p)
            print(f"📁 Copied to conversation artifact: {p}")

    # Open image in Preview so user sees the live overlay
    subprocess.run(["open", annotated_path])
    print("🚀 Opened visual debug overlay in macOS Preview!\n")

if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        pass
