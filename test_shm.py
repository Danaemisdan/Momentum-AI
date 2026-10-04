import sys
import io
from multiprocessing import shared_memory
import numpy as np
from PIL import Image

try:
    shm = shared_memory.SharedMemory(name='momentum_vision_shm')
    width = int.from_bytes(shm.buf[0:4], byteorder=sys.byteorder)
    height = int.from_bytes(shm.buf[4:8], byteorder=sys.byteorder)
    
    print(f"SHM Width: {width}, Height: {height}")
    
    expected_size = width * height * 4
    buffer = shm.buf[8:8+expected_size]
    
    arr = np.ndarray((height, width, 4), dtype=np.uint8, buffer=buffer)
    print(f"SHM Array Mean: {np.mean(arr)}")
    
    img = Image.fromarray(arr, 'RGBA').convert('RGB')
    img.save("debug_shm.jpg", format='JPEG')
    print("Saved debug_shm.jpg")
    
    shm.close()
except Exception as e:
    print(f"Error: {e}")
