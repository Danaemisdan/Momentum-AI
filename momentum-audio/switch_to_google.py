import re

with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Remove the faster-whisper load
whisper_load = """    print("\ns [2/2] Initializing Faster-Whisper (Blazing Fast Local STT with VAD)...")
    try:
        app_state["whisper"] = WhisperModel("tiny.en", device="cpu", compute_type="int8")
        print("o. Faster-Whisper ?" Ready.")
    except Exception as e:
        print(f"s,? Failed to load Faster-Whisper: {e}")"""
content = content.replace(whisper_load, """    print("\ns [2/2] Initializing Google Web Speech API (Blazing Fast Cloud STT)...")""")

# Replace the STT callback
stt_callback_pattern = re.compile(r'def stt_callback\(recognizer, audio\):.*?# Filter out pure filler sounds', re.DOTALL)
new_stt_callback = """def stt_callback(recognizer, audio):
        try:
            # Transcribe with Google Web Speech API (Free, super fast, high accuracy)
            text = recognizer.recognize_google(audio).strip()
            
            if text:
                # Filter out notorious Whisper hallucinations on silence
                clean_lower = re.sub(r'[^a-z]', '', text.lower())
                hallucinations = {
                    "thankyou", "you", "mmhmm", "mm", "hmm", "okay", "yeah", "thatsthat", 
                    "god", "thatsit", "shh", "thisiscrazy", "oh", "ew", "hahaha", "ha", 
                    "huh", "wow", "thanks", "byebye", "bye", "yourewelcome", 
                    "welcome", "hello", "hi", "test", "testing", "imsorry", "sorry", 
                    "yep", "yes", "nah", "uh", "um", "ah", "i", "a", "so", 
                    "right", "sure", "well", "and", "but", "or", "to", "the"
                }
                
                # Filter out pure filler sounds"""

content = re.sub(stt_callback_pattern, new_stt_callback, content)

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
