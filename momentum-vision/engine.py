import os
import base64
import io
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Optional
from llama_cpp import Llama
from llama_cpp.llama_chat_format import Llava15ChatHandler
import uvicorn
from contextlib import asynccontextmanager
from multiprocessing import shared_memory
import numpy as np
from PIL import Image

# Global model instance
llm = None

# Screen dimensions (will be sent by Rust or hardcoded for now)
# Rust will write RGBA bytes. 
SHM_NAME = "momentum_vision_shm_v2"

@asynccontextmanager
async def lifespan(app: FastAPI):
    global llm
    model_path = os.path.join(os.path.dirname(__file__), "momentum-vision-engine.gguf")
    projector_path = os.path.join(os.path.dirname(__file__), "momentum-vision-projector.gguf")
    
    if not os.path.exists(model_path):
        print(f"ERROR: Model not found at {model_path}")
    elif not os.path.exists(projector_path):
        print(f"ERROR: Projector not found at {projector_path}")
    else:
        print(f"Loading Vision Model from {model_path} with Metal Acceleration...")
        try:
            chat_handler = Llava15ChatHandler(clip_model_path=projector_path)
            llm = Llama(
                model_path=model_path,
                chat_handler=chat_handler,
                n_gpu_layers=-1, # Offload entirely to Metal
                n_threads=3,     # Prevent CPU starvation
                n_ctx=4096,      # Context window for image tokens
                verbose=False
            )
            print("Vision Model Loaded Successfully!")
        except Exception as e:
            print(f"Failed to load model: {e}")
            
    yield
    print("Shutting down Vision Engine...")

app = FastAPI(lifespan=lifespan)

class VisionRequest(BaseModel):
    use_shm: bool = True
    width: int = 1920
    height: int = 1080
    prompt: str = "Describe what you see on the screen and list all interactive elements."
    max_tokens: int = 512
    crop_x: Optional[int] = None
    crop_y: Optional[int] = None
    crop_w: Optional[int] = None
    crop_h: Optional[int] = None

@app.post("/perceive")
async def perceive(request: VisionRequest):
    if llm is None:
        raise HTTPException(status_code=500, detail="Vision model not loaded.")
        
    image_b64 = ""
    
    if request.use_shm:
        try:
            from multiprocessing import shared_memory
            import sys
            import tempfile
            import os
            
            temp_path = os.path.join(tempfile.gettempdir(), "momentum_vision.bin")
            with open(temp_path, "r") as f:
                real_name = f.read().strip()
                if real_name.startswith('/'):
                    real_name = real_name[1:]
            
            # Connect to the exact POSIX block the Rust agent auto-generated
            import os
            if os.name == 'nt':
                try:
                    shm = shared_memory.SharedMemory(name=real_name)
                except FileNotFoundError:
                    try:
                        shm = shared_memory.SharedMemory(name=f"Local\\{real_name}")
                    except FileNotFoundError:
                        shm = shared_memory.SharedMemory(name=f"Global\\{real_name}")
            else:
                shm = shared_memory.SharedMemory(name=real_name)
            
            # Read dynamic width and height from the first 8 bytes
            width = int.from_bytes(shm.buf[0:4], byteorder=sys.byteorder)
            height = int.from_bytes(shm.buf[4:8], byteorder=sys.byteorder)
            
            if width <= 0 or height <= 0 or width > 10000 or height > 10000:
                raise HTTPException(status_code=500, detail=f"Invalid SHM dimensions: {width}x{height}")
            
            # Read RGBA raw pixels (width * height * 4 bytes) starting at byte 8
            expected_size = width * height * 4
            buffer = shm.buf[8:8+expected_size]
            
            # Convert to numpy array and deep copy to detach from the SHM buffer
            arr = np.ndarray((height, width, 4), dtype=np.uint8, buffer=buffer).copy()
            
            print(f"DEBUG VISION: {width}x{height}, Mean: {np.mean(arr)}", flush=True)
            
            # Close SHM connection (don't unlink, Rust owns it)
            del buffer
            shm.close()
            
            # Apply foveated cropping if requested
            if request.crop_w is not None and request.crop_h is not None and request.crop_x is not None and request.crop_y is not None:
                x1 = max(0, request.crop_x)
                y1 = max(0, request.crop_y)
                x2 = min(width, x1 + request.crop_w)
                y2 = min(height, y1 + request.crop_h)
                if x2 > x1 and y2 > y1:
                    arr = arr[y1:y2, x1:x2]
            
            # Convert to PIL Image (RGBA -> RGB)
            img = Image.fromarray(arr, 'RGBA').convert('RGB')
            
            # Downsize image to prevent llama.cpp segfaults and severely cut down processing time (1 minute -> 5 seconds)
            MAX_DIM = 512
            if img.width > MAX_DIM or img.height > MAX_DIM:
                img.thumbnail((MAX_DIM, MAX_DIM), Image.Resampling.LANCZOS)
            
            # Convert image to base64
            img_byte_arr = io.BytesIO()
            img.save(img_byte_arr, format='JPEG', quality=85)
            image_b64 = base64.b64encode(img_byte_arr.getvalue()).decode('utf-8')
            
        except Exception as e:
            import traceback
            traceback.print_exc()
            raise HTTPException(status_code=500, detail=f"Failed to read from Shared Memory: {e}")
    else:
        raise HTTPException(status_code=400, detail="Fallback base64 not provided. Must use_shm=True")

    try:
        response = llm.create_chat_completion(
            messages=[
                {
                    "role": "user",
                    "content": [
                        {"type": "image_url", "image_url": {"url": f"data:image/jpeg;base64,{image_b64}"}},
                        {"type": "text", "text": request.prompt}
                    ]
                }
            ],
            max_tokens=request.max_tokens,
            temperature=0.1
        )
        
        return {
            "status": "success",
            "text": response["choices"][0]["message"]["content"]
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/health")
async def health():
    return {"status": "ok", "model_loaded": llm is not None}

if __name__ == "__main__":
    uvicorn.run("engine:app", host="127.0.0.1", port=8001, reload=True)
