from faster_whisper import WhisperModel
import numpy as np

model = WhisperModel("tiny.en", device="cpu", compute_type="int8")

audio = np.zeros(16000*2, dtype=np.float32)
# With initial prompt to encourage non-speech tokens
segments, info = model.transcribe(audio, initial_prompt="[laughs] [coughs] [sneezes] [crying] [angry]", condition_on_previous_text=False)
text = " ".join([s.text for s in segments])
print("Transcribed:", text)
