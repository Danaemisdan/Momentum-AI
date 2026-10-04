import io
import re
import asyncio
import numpy as np
import soundfile as sf
from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from contextlib import asynccontextmanager
from concurrent.futures import ThreadPoolExecutor
from kokoro_onnx import Kokoro
import speech_recognition as sr
import sounddevice as sd
import json
import queue
import threading
import re
import difflib
from collections import deque
from faster_whisper import WhisperModel

audio_playback_queue = queue.Queue()

def playback_worker():
    while True:
        data = audio_playback_queue.get()
        if data is None:
            break
        samples_t, sample_rate, sentence = data
        try:
            # Broadcast caption before native playback
            if app_state["loop"] is not None:
                dead_ws = set()
                for ws in list(app_state["tts_websockets"]):
                    try:
                        asyncio.run_coroutine_threadsafe(
                            ws.send_text(json.dumps({"type": "caption", "text": sentence})),
                            app_state["loop"]
                        )
                    except Exception:
                        dead_ws.add(ws)
                app_state["tts_websockets"].difference_update(dead_ws)
                
            sd.play(samples_t, sample_rate)
            sd.wait()
        except Exception as e:
            print(f"🚨 Playback error: {e}")


app_state = {
    "kokoro": None,
    "executor": None,
    "stt_websockets": set(),
    "tts_websockets": set(),
    "loop": None,
    "whisper": None,
    "recent_agent_speech": deque(maxlen=10),
}

# ─── Sentence splitter ────────────────────────────────────────────────────────
_SENT_RE = re.compile(r'(?<=[.!?…])\s+')

def split_sentences(text: str) -> list[str]:
    """Split text into synthesizable chunks. Min 3 chars."""
    chunks = _SENT_RE.split(text.strip())
    out = []
    buf = ""
    for chunk in chunks:
        buf = (buf + " " + chunk).strip() if buf else chunk
        if len(buf) >= 3:
            out.append(buf)
            buf = ""
    if buf:
        out.append(buf)
    return out or [text]

from pedalboard import Pedalboard, Reverb, Compressor, HighpassFilter, PitchShift

# ─── Vocal Processing Matrix ──────────────────────────────────────────────────
# Lightweight, humanizing DSP chain for the "bot presence" feel
vocal_board = Pedalboard([
    Compressor(threshold_db=-20.0, ratio=3.0, attack_ms=5.0, release_ms=50.0),
])

def synth_chunk(kokoro, text: str, voice: str):
    """Synthesize one sentence, apply DSP, and return raw WAV bytes natively."""
    text = text.strip()
    if not text:
        return b"", None, None
    samples, sample_rate = kokoro.create(text, voice=voice, speed=1.05, lang="en-us")
    
    # Process through the pedalboard pipeline (fast, native C++)
    processed_samples = vocal_board(samples, sample_rate)
    
    buf = io.BytesIO()
    sf.write(buf, processed_samples, sample_rate, format='WAV', subtype='FLOAT')
    return buf.getvalue(), processed_samples.T, sample_rate

@asynccontextmanager
async def lifespan(app: FastAPI):
    threading.Thread(target=playback_worker, daemon=True).start()
    print("\n⚡ [1/2] Loading Kokoro-82M ONNX TTS Engine...")
    try:
        app_state["kokoro"] = Kokoro("kokoro-v1.0.onnx", "voices-v1.0.bin")
        app_state["executor"] = ThreadPoolExecutor(max_workers=2)
        print("✅ Kokoro TTS — Lightning-Fast Native Pipeline Active. Sub-200ms latency enabled.")
    except Exception as e:
        print(f"⚠️ Failed to load Kokoro: {e}")
    print("⚡ [2/2] Initializing DSP ThreadPool (2 Threads)...")
    app_state["executor"] = ThreadPoolExecutor(max_workers=2)
    
    # ─── Global STT Engine ────────────────────────────────────────────────────────
    print("⚡ [3/3] Initializing Global STT Engine (Google Speech)...")
    # app_state["whisper"] = WhisperModel("small.en", device="cpu", compute_type="int8", cpu_threads=4)
    r = sr.Recognizer()
    r.pause_threshold = 1.5
    r.non_speaking_duration = 0.5
    
    try:
        mic = sr.Microphone()
    except OSError:
        mic = sr.Microphone(device_index=None)

    loop = asyncio.get_event_loop()
    app_state["loop"] = loop

    def stt_callback(recognizer, audio):
        try:
            try:
                text = recognizer.recognize_google(audio)
            except sr.UnknownValueError:
                return # Ignore static/silence completely
            
            if text:
                # Filter out notorious Whisper hallucinations on silence
                clean_lower = re.sub(r'[^a-z]', '', text.lower())
                hallucinations = {"thankyou", "you", "mmhmm", "mm", "hmm", "okay", "yeah", "thatsthat", "god", "thatsit", "shh", "thisiscrazy", "oh", "ew", "hahaha", "ha", "huh", "wow", "what"}
                if clean_lower in hallucinations:
                    return

                # Text-based Acoustic Echo Cancellation (AEC)
                # Prevent the microphone from hearing the agent's own speaker output
                is_echo = False
                for recent in app_state["recent_agent_speech"]:
                    if difflib.SequenceMatcher(None, text.lower(), recent.lower()).ratio() > 0.6:
                        is_echo = True
                        break
                
                if is_echo:
                    print(f"🔇 AEC Filtered Echo: {text}", flush=True)
                    return

                # INTERRUPT AUDIO! Only halt playback if user actually spoke words (not just a cough/laugh)
                if not re.match(r'^\[.*\]$', text):
                    with audio_playback_queue.mutex:
                        audio_playback_queue.queue.clear()
                    sd.stop()

                print(f"🎙️ STT Heard: {text}", flush=True)
                # Broadcast to all connected UI clients
                dead_ws = set()
                for ws in list(app_state["stt_websockets"]):
                    try:
                        asyncio.run_coroutine_threadsafe(
                            ws.send_text(json.dumps({"transcript": text, "isFinal": True})), 
                            loop
                        )
                    except Exception:
                        dead_ws.add(ws)
                app_state["stt_websockets"].difference_update(dead_ws)
        except Exception:
            pass

    stop_listening = r.listen_in_background(mic, stt_callback, phrase_time_limit=5)
    
    yield
    print("Shutting down core systems...")
    stop_listening(wait_for_stop=False)
    if app_state["executor"]:
        app_state["executor"].shutdown(wait=False)


app = FastAPI(lifespan=lifespan)

@app.get("/health")
async def health():
    return {"status": "ok"}

@app.websocket("/tts")
async def audio_stream(websocket: WebSocket):
    await websocket.accept()
    print("🔌 UI Audio Stream Connected.", flush=True)
    app_state["tts_websockets"].add(websocket)

    voice_name = "af_heart"
    loop = asyncio.get_event_loop()
    kokoro = app_state["kokoro"]
    executor = app_state["executor"]

    try:
        while True:
            text_data = await websocket.receive_text()
            if not text_data.strip():
                continue

            # Split into sentences for streaming synthesis
            sentences = split_sentences(text_data)
            print(f"🎙️ Synth {len(sentences)} chunk(s): {text_data[:60]}...", flush=True)

            for sentence in sentences:
                app_state["recent_agent_speech"].append(sentence)

            # Synthesize the FIRST sentence immediately (blocking in thread)
            # then fire subsequent ones as they finish — ultra-low first-audio latency
            for i, sentence in enumerate(sentences):
                if not sentence.strip():
                    continue
                try:
                    wav_bytes, samples_t, sample_rate = await loop.run_in_executor(
                        executor,
                        synth_chunk,
                        kokoro, sentence, voice_name
                    )
                    if samples_t is not None:
                        audio_playback_queue.put((samples_t, sample_rate, sentence))
                    await websocket.send_bytes(wav_bytes)
                except Exception as e:
                    print(f"🚨 Synthesis fault on chunk {i}: {e}")

    except WebSocketDisconnect:
        print("🔌 UI Audio Stream Terminated.")
    except Exception as e:
        print(f"🚨 WebSocket error: {e}")
    finally:
        app_state["tts_websockets"].discard(websocket)

@app.websocket("/stt")
async def stt_stream(websocket: WebSocket):
    await websocket.accept()
    print("🔌 UI STT Stream Connected.", flush=True)
    app_state["stt_websockets"].add(websocket)
    try:
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        pass
    finally:
        app_state["stt_websockets"].discard(websocket)
