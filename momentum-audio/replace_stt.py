import re, json
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('from faster_whisper import WhisperModel', 'from vosk import Model, KaldiRecognizer\nimport queue')

parts1 = content.split('# ─── Global STT Engine ────────────────────────────────────────────────────────')
parts2 = parts1[1].split('    yield\n    print("Shutting down core systems...")')

new_stt = """
    print("⚡ [3/3] Initializing Global STT Engine (Vosk Streaming)...")
    app_state["vosk_model"] = Model("vosk-model")
    
    loop = asyncio.get_event_loop()
    app_state["loop"] = loop
    audio_q = queue.Queue()

    def audio_callback(indata, frames, time, status):
        audio_q.put(bytes(indata))

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
                    print(f"🎙️ STT Heard (Final): {text}", flush=True)
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

    import threading
    stt_thread = threading.Thread(target=stt_worker, daemon=True)
    stt_thread.start()
    
    yield
    print("Shutting down core systems...")
"""

new_content = parts1[0] + '# ─── Global STT Engine (Vosk Streaming) ────────────────────────────────────────────────\n' + new_stt + parts2[1]
new_content = new_content.replace('stop_listening(wait_for_stop=False)', 'app_state["shutdown_event"] = True')

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(new_content)
