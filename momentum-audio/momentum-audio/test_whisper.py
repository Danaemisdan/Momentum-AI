from faster_whisper import WhisperModel
import numpy as np

print("Loading model...")
model = WhisperModel("tiny.en", device="cpu", compute_type="int8")
print("Model loaded.")

# create dummy audio
audio = np.zeros(16000, dtype=np.float32)
segments, info = model.transcribe(audio, beam_size=1)
text = " ".join([s.text for s in segments])
print("Transcribed:", text)
