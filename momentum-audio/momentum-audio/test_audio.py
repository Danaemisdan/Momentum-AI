import sys
import numpy as np
import sounddevice as sd
from kokoro_onnx import Kokoro
from pedalboard import Pedalboard, Compressor

print("Loading Kokoro...")
kokoro = Kokoro("kokoro-v1.0.onnx", "voices-v1.0.bin")
print("Synthesizing...")
samples, sr = kokoro.create("Testing native audio playback on macOS.", voice="af_heart", speed=1.05)

print(f"Original shape: {samples.shape}")

board = Pedalboard([Compressor(threshold_db=-20.0, ratio=3.0, attack_ms=5.0, release_ms=50.0)])
processed = board(samples, sr)

print(f"Pedalboard shape: {processed.shape}")
print(f"Transposed shape: {processed.T.shape}")

print("Playing natively...")
try:
    sd.play(processed.T, sr)
    sd.wait()
    print("Success.")
except Exception as e:
    print(f"Error playing: {e}")
