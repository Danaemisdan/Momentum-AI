import numpy as np
import soundfile as sf
from kokoro_onnx import Kokoro
import sys

print("Loading Kokoro...")
try:
    kokoro = Kokoro("kokoro-v1.0.onnx", "voices-v1.0.bin")
    print("Success! Synthesizing...")
    samples, sample_rate = kokoro.create("Hello world.", voice="af_heart", speed=1.0, lang="en-us")
    print(f"Generated {len(samples)} samples at {sample_rate} Hz")
    sys.exit(0)
except Exception as e:
    print(f"Failed: {e}")
    sys.exit(1)
