import numpy as np
from pedalboard import Pedalboard, PitchShift, Chorus, Compressor

board = Pedalboard([
    Compressor(threshold_db=-15, ratio=3),
    Chorus(rate_hz=1.0, depth=0.1, centre_delay_ms=7.0),
    PitchShift(semitones=-1.5)
])

# Dummy array of 1 second raw 24000Hz PCM
samples = np.random.randn(24000).astype(np.float32)

# Pedalboard expects (channels, samples)
samples_2d = np.expand_dims(samples, axis=0)
effected = board(samples_2d, 24000)
print("Effected output shape:", effected.shape)
