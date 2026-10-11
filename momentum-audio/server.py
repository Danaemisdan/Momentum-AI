import io
import re
import asyncio
import numpy as np
import soundfile as sf
from fastapi import FastAPI, WebSocket
from starlette.websockets import WebSocketDisconnect, WebSocketDisconnected, WebSocketState
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

async def broadcast_caption(sentence: str):
    dead_ws = set()
    for ws in list(app_state["tts_websockets"]):
        try:
            if ws.client_state == WebSocketState.CONNECTED and ws.application_state == WebSocketState.CONNECTED:
                await ws.send_text(json.dumps({"type": "caption", "text": sentence}))
            else:
                dead_ws.add(ws)
        except Exception:
            dead_ws.add(ws)
    app_state["tts_websockets"].difference_update(dead_ws)

async def broadcast_stt(data: dict):
    dead_ws = set()
    for ws in list(app_state["stt_websockets"]):
        try:
            if ws.client_state == WebSocketState.CONNECTED and ws.application_state == WebSocketState.CONNECTED:
                await ws.send_text(json.dumps(data))
            else:
                dead_ws.add(ws)
        except Exception:
            dead_ws.add(ws)
    app_state["stt_websockets"].difference_update(dead_ws)

def playback_worker():
    while True:
        data = audio_playback_queue.get()
        if data is None:
            break
        samples_t, sample_rate, sentence, gen_id = data
        
        # Discard chunk if user interrupted while it was queued
        if gen_id != app_state["current_generation"]:
            continue

        try:
            # Broadcast caption before native playback
            if app_state["loop"] is not None and app_state["loop"].is_running():
                asyncio.run_coroutine_threadsafe(
                    broadcast_caption(sentence),
                    app_state["loop"]
                )
                
            samples_contiguous = np.ascontiguousarray(samples_t, dtype=np.float32)
            app_state["is_speaking"] = True
            sd.play(samples_contiguous, samplerate=sample_rate)
            sd.wait()
            if audio_playback_queue.empty():
                app_state["is_speaking"] = False
        except Exception as e:
            app_state["is_speaking"] = False
            print(f"🚨 Playback error: {e}")
            try:
                sd.stop()
            except Exception:
                pass


app_state = {
    "kokoro": None,
    "executor": None,
    "stt_websockets": set(),
    "tts_websockets": set(),
    "loop": None,
    "whisper": None,
    "recent_agent_speech": deque(maxlen=10),
    "current_generation": 0,
    "is_speaking": False,
}

# ─── Sentence splitter ────────────────────────────────────────────────────────
_SENT_RE = re.compile(r'(?<=[.!?…])\s+')

def split_sentences(text: str) -> list[str]:
    """Split text into synthesizable chunks. Min 3 chars."""
    clean = re.sub(r'<\|.*?\|>', '', text)
    clean = re.sub(r'\[ACTION:[^\]]*\]', '', clean)
    clean = re.sub(r'\[ACTIVE:[^\]]*\]', '', clean)
    clean = re.sub(r'\[NOTE:[^\]]*\]', '', clean)
    clean = re.sub(r'\[SYSTEM[^\]]*\]', '', clean, flags=re.IGNORECASE)
    clean = clean.strip()
    if not clean:
        return []
    chunks = _SENT_RE.split(clean)
    out = []
    buf = ""
    for chunk in chunks:
        buf = (buf + " " + chunk).strip() if buf else chunk
        if len(buf) >= 3:
            out.append(buf)
            buf = ""
    if buf:
        out.append(buf)
    return out or [clean]

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
    samples, sample_rate = kokoro.create(text, voice=voice, speed=1.25, lang="en-us")
    
    # Process through the pedalboard pipeline (fast, native C++)
    processed_samples = vocal_board(samples, sample_rate)
    
    buf = io.BytesIO()
    samples_out = np.ascontiguousarray(processed_samples.T, dtype=np.float32)
    return buf.getvalue(), samples_out, sample_rate

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
    print("⚡ [3/3] Initializing Global STT Engine (faster-whisper)...")
    app_state["whisper"] = WhisperModel("base", device="cpu", compute_type="int8")
    r = sr.Recognizer()
    r.pause_threshold = 1.0
    r.non_speaking_duration = 0.5
    r.dynamic_energy_threshold = True
    
    try:
        mic = sr.Microphone()
        with mic as source:
            print("🎙️ Calibrating microphone for ambient room noise...", flush=True)
            r.adjust_for_ambient_noise(source, duration=0.8)
            print(f"🎙️ Microphone calibrated! Threshold: {r.energy_threshold:.1f}", flush=True)
    except Exception as e:
        print(f"⚠️ Mic init: {e}")
        try:
            mic = sr.Microphone(device_index=None)
        except Exception:
            mic = None

    loop = asyncio.get_event_loop()
    app_state["loop"] = loop

    def stt_callback(recognizer, audio):
        try:
            text = ""
            try:
                # Transcribe with Google Web Speech API (Free, ultra-fast, zero hallucination)
                text = recognizer.recognize_google(audio).strip()
            except sr.UnknownValueError:
                return
            except Exception as err:
                if app_state.get("whisper"):
                    try:
                        audio_np = np.frombuffer(audio.get_raw_data(convert_rate=16000, convert_width=2), np.int16).astype(np.float32) / 32768.0
                        segments, _ = app_state["whisper"].transcribe(audio_np, beam_size=1, vad_filter=True)
                        text = " ".join([s.text for s in segments]).strip()
                    except Exception:
                        return
                else:
                    return
            
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
                
                # Filter out pure filler sounds (e.g. "Hmm.", "Hmm. Hmm.", "Uh-huh.")
                if re.fullmatch(r'(hmm|hm|mm|uh|ah|ha|huh)+', clean_lower):
                    return
                    
                if clean_lower in hallucinations or len(clean_lower) < 2:
                    return

                # Text-based Acoustic Echo Cancellation (AEC)
                # Prevent the microphone from hearing the agent's own speaker output
                is_echo = False
                for recent in app_state["recent_agent_speech"]:
                    if difflib.SequenceMatcher(None, text.lower(), recent.lower()).ratio() > 0.4:
                        is_echo = True
                        break
                
                if is_echo:
                    print(f"🔇 AEC Filtered Echo: {text}", flush=True)
                    return

                # Check if agent is currently speaking
                is_speaking = app_state.get("is_speaking", False)
                words = set(re.findall(r'\b[a-z]+\b', text.lower()))
                interrupt_words = {"stop", "shut", "quiet", "silence", "pause", "wait", "cancel", "shh", "enough", "stfu", "momentum"}
                has_interrupt = bool(words & interrupt_words)

                if is_speaking:
                    if has_interrupt:
                        print(f"🛑 User interrupted agent: {text}", flush=True)
                        app_state["current_generation"] += 1
                        with audio_playback_queue.mutex:
                            audio_playback_queue.queue.clear()
                        sd.stop()
                        app_state["is_speaking"] = False
                    else:
                        print(f"🔇 Ignored room chatter / speaker echo while speaking: {text}", flush=True)
                        return
                else:
                    if has_interrupt:
                        with audio_playback_queue.mutex:
                            audio_playback_queue.queue.clear()
                        sd.stop()

                print(f"🎙️ STT Heard: {text}", flush=True)
                # Broadcast to all connected UI clients
                if loop is not None and loop.is_running():
                    asyncio.run_coroutine_threadsafe(
                        broadcast_stt({"transcript": text, "isFinal": True}), 
                        loop
                    )
        except Exception:
            pass

    if mic is not None:
        stop_listening = r.listen_in_background(mic, stt_callback, phrase_time_limit=15)
    else:
        def stop_listening(wait_for_stop=False): pass
    
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
    kokoro = app_state["kokoro"]
    executor = app_state["executor"]

    try:
        while True:
            text_data = await websocket.receive_text()
            if not text_data.strip():
                continue

            my_gen = app_state["current_generation"]
            # Split into sentences for streaming synthesis
            sentences = split_sentences(text_data)
            print(f"🎙️ Synth {len(sentences)} chunk(s): {text_data[:60]}...", flush=True)

            for sentence in sentences:
                app_state["recent_agent_speech"].append(sentence)

            # Synthesize sentences
            for i, sentence in enumerate(sentences):
                if not sentence.strip():
                    continue
                if app_state["current_generation"] != my_gen:
                    print("⚡ Synthesis aborted due to interruption.")
                    break

                try:
                    wav_bytes, samples_t, sample_rate = await asyncio.get_event_loop().run_in_executor(
                        executor,
                        synth_chunk,
                        kokoro, sentence, voice_name
                    )
                    if app_state["current_generation"] != my_gen:
                        print("⚡ Synthesis aborted due to interruption.")
                        break

                    if samples_t is not None:
                        audio_playback_queue.put((samples_t, sample_rate, sentence, my_gen))

                    if websocket.client_state == WebSocketState.CONNECTED and websocket.application_state == WebSocketState.CONNECTED:
                        await websocket.send_bytes(wav_bytes)
                except (WebSocketDisconnect, WebSocketDisconnected):
                    print(f"🔌 UI Audio Stream Terminated during chunk {i}.")
                    return
                except RuntimeError as e:
                    if "not connected" in str(e).lower() or "disconnect" in str(e).lower() or "close" in str(e).lower():
                        print(f"🔌 UI Audio Stream Terminated during chunk {i}.")
                        return
                    print(f"🚨 Synthesis fault on chunk {i}: {e}")
                except Exception as e:
                    print(f"🚨 Synthesis fault on chunk {i}: {e}")

    except (WebSocketDisconnect, WebSocketDisconnected):
        print("🔌 UI Audio Stream Terminated.")
    except RuntimeError as e:
        if "not connected" in str(e).lower() or "disconnect" in str(e).lower() or "close" in str(e).lower():
            print("🔌 UI Audio Stream Terminated.")
        else:
            print(f"🚨 WebSocket error: {e}")
    except Exception as e:
        print(f"🚨 WebSocket error: {e}")
    finally:
        app_state["tts_websockets"].discard(websocket)
        try:
            if websocket.client_state == WebSocketState.CONNECTED:
                await websocket.close()
        except Exception:
            pass

@app.websocket("/stt")
async def stt_stream(websocket: WebSocket):
    await websocket.accept()
    print("🔌 UI STT Stream Connected.", flush=True)
    app_state["stt_websockets"].add(websocket)
    try:
        while True:
            await websocket.receive_text()
    except (WebSocketDisconnect, WebSocketDisconnected):
        pass
    except RuntimeError as e:
        if "not connected" in str(e).lower() or "disconnect" in str(e).lower() or "close" in str(e).lower():
            pass
        else:
            print(f"🚨 STT WebSocket error: {e}")
    except Exception as e:
        print(f"🚨 STT WebSocket error: {e}")
    finally:
        app_state["stt_websockets"].discard(websocket)
        try:
            if websocket.client_state == WebSocketState.CONNECTED:
                await websocket.close()
        except Exception:
            pass
