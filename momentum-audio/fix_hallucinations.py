import re
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Add aggressive hallucination filtering
old_filter = '''            # Filter hallucinations
            lower_text = text.lower().replace(".", "").replace(",", "").replace("!", "").replace("?", "").strip()
            if lower_text in ["you", "thank you", "bye", "okay", "ok", "mm-hmm", "mmm", "hmm"]:
                return'''
new_filter = '''            # Filter hallucinations
            lower_text = text.lower().replace(".", "").replace(",", "").replace("!", "").replace("?", "").strip()
            
            # Whisper hallucination patterns (YouTube/Static)
            hallucinations = [
                "you", "thank you", "bye", "okay", "ok", "mm-hmm", "mmm", "hmm",
                "thanks for watching", "thank you for watching", "please subscribe",
                "subscribe to the channel", "amaraorg", "amara", "let us know in the comments",
                "in the comments below", "see you next time", "see you in the next one",
                "i'll take a look at the next one", "i'll see you in the next one"
            ]
            
            if lower_text in hallucinations or any(h in lower_text for h in ["subscribe", "thanks for watching", "comments below"]):
                return'''
content = content.replace(old_filter, new_filter)

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
