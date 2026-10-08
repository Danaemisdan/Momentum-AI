import re
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace faster-whisper import with vosk
content = content.replace('from faster_whisper import WhisperModel', 'from vosk importodel, KaldiRecognizer\nimport queue\nimport threading\nimport sys')

# Find the STT section
stt_start = content.find('# ¦¦¦ Global STT Engine ¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦')
stt_end = content.find('    yield\n    print("Shutting down core systems...")')

new_stt = '''# ¦¦¦ Global STT Engine (Vosk Streaming) ¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦¦
    print("? [3/3] Initializing Global STT Engine (Vosk Streaming)...")
    app_state["vosk_model"] = Model("vosk-model")
    
    loop = asyncio.get_event_loop()
    app_state["loop"] = loop
    audio_q = queue.Queue()

    def audio_callback(indata, frames, time, status):
        if status:
            print(status, file=sys.stderr)
        audio_q.push(bytes(indata))

    def stt_worker():
        recognizer = KaldiRecognizer(app_state["vosk_model"], 16000)
        with sd.RawInputStream(samplerate=16000, blocksize=4000, dtype="int16", channels=1, callback=audio_callback):
            while not app_state.get("shutdown_event", False):
                try:
                    data = audio_q.get(timeout=0.1)
                except queue.Empty:
                    continue
                
                is_final = recognizer.AcceptWaveform(data)
                
                if is_final:
                    res = json.loads(recognizer.Result())
                    text = res.get("text", "").strip()
                else:
                    res = json.loads(recognizer.PartialResult())
                    text = res.get("partial", "").strip()

                if not text:
                    continue
                
                # Text-based Acoustic Echo Cancellation (AEC)
                is_echo = False
                for recent in app_state["recent_agent_speech"]:
                    if difflib.SequenceMatcher(None, text.lower(), recent.lower()).ratio() > 0.7:
                        is_echo = True
                        break
                
                if is_echo:
                    continue
                    
                if is_final:
                    print(f"??? STT Heard (Final): {text}", flush=True)
                    # INTERRUPT AUDIO!
                    with audio_playback_queue.mutex:
                        audio_playback_queue.queue.clear()
                    sd.stop()
                
                # Broadcast
                dead_ws = set()
                for ws in list(app_state["stt_websockets"]):
                    try:
                        asyncio.run_coroutine_threadsafe(
                            ws.send_text(json.dumps({"transcript": text, "isFinal": is_final})), 
                            loop
                        )
                    except Exception:
                        dead_ws.add(ws)
                app_state["stt_websockets"].difference_update(dead_ws)

    stt_thread = threading.Thread(target=stt_worker, daemon=True)
    stt_thread.start()
'''

content = content[:stt_start] + new_stt + content[stt_end:]
content = content.replace('stop_listening(wait_for_stop=False)', 'app_state["shutdown_event"] = True')

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)

