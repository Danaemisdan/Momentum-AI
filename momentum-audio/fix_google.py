import re
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace faster-whisper logic with speech_recognition Google API
whisper_import = "from faster_whisper import WhisperModel"
google_import = ""
content = content.replace(whisper_import, google_import)

whisper_init = """    print("Initializing Faster-Whisper (Blazing Fast Local STT with VAD)...")
    app_state["whisper_model"] = WhisperModel("small.en", device="cpu", compute_type="int8")"""
google_init = """    print("Initializing Google Cloud Speech API (Ultra-Accurate, Zero CPU)...")"""
content = content.replace(whisper_init, google_init)

whisper_stt = """            wav_data = audio.get_wav_data()
            segments, info = app_state["whisper_model"].transcribe(io.BytesIO(wav_data), beam_size=1, vad_filter=True, vad_parameters=dict(min_speech_duration_ms=250))
            text = " ".join([segment.text for segment in segments]).strip()"""
google_stt = """            # Use Google Web Speech API for zero CPU usage, instant response, and high accuracy
            text = recognizer.recognize_google(audio).strip()"""
content = content.replace(whisper_stt, google_stt)

# Fix pause threshold to stop cutting off
content = content.replace("r.pause_threshold = 0.5", "r.pause_threshold = 1.2")
content = content.replace("r.non_speaking_duration = 0.3", "r.non_speaking_duration = 1.0")

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
