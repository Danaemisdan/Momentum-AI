import re
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace Google API with faster-whisper base.en
google_init = '    print("Initializing Google Cloud Speech API (Ultra-Accurate, Zero CPU)...")'
whisper_init = """    print("Initializing Faster-Whisper (Blazing Fast Local STT with VAD)...")
    from faster_whisper import WhisperModel
    app_state["whisper_model"] = WhisperModel("base.en", device="cpu", compute_type="int8", cpu_threads=4)"""
content = content.replace(google_init, whisper_init)

google_stt = """            # Use Google Web Speech API for zero CPU usage, instant response, and high accuracy
            text = recognizer.recognize_google(audio).strip()"""
whisper_stt = """            wav_data = audio.get_wav_data()
            segments, info = app_state["whisper_model"].transcribe(
                io.BytesIO(wav_data), 
                beam_size=2, 
                vad_filter=True, 
                vad_parameters=dict(min_speech_duration_ms=250),
                condition_on_previous_text=False
            )
            text = " ".join([segment.text for segment in segments]).strip()
            # Filter hallucinations
            lower_text = text.lower().replace(".", "").replace(",", "").replace("!", "").replace("?", "").strip()
            if lower_text in ["you", "thank you", "bye", "okay", "ok", "mm-hmm", "mmm", "hmm"]:
                return"""
content = content.replace(google_stt, whisper_stt)

# Suppress traceback printing to avoid spam
content = content.replace("traceback.print_exc()", "pass")

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
