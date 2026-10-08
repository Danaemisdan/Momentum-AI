import re
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Find the VAD block
vad_block = """                text = segment.text.strip()
                if text:"""
new_vad_block = """                text = segment.text.strip()
                
                # Anti-hallucination filters for Faster Whisper on silence/static
                lower_text = text.lower()
                hallucination_triggers = [
                    "subscribe",
                    "thanks for watching",
                    "let us know in the comments below",
                    "thank you for watching",
                    "comments below",
                    "like and subscribe",
                    "mm-hmm",
                    "you"
                ]
                
                # Check if the text is exactly or heavily composed of hallucinations
                is_hallucination = False
                if len(text) < 40:
                    for trigger in hallucination_triggers:
                        if trigger in lower_text:
                            is_hallucination = True
                            break
                            
                # Single word weirdness filter
                if len(text.split()) <= 2 and (lower_text.strip('.?,! ') in ['you', 'yeah', 'uh', 'um', 'mm']):
                    is_hallucination = True
                            
                if text and not is_hallucination:"""

content = content.replace(vad_block, new_vad_block)

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
