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
        print("Hardware Optimization Mode Active: Vision Model will be loaded dynamically on demand to preserve RAM.")
            
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

        
    image_b64 = ""
    
    if request.use_shm:
        try:
            import tempfile
            import os
            
            temp_path = os.path.join(tempfile.gettempdir(), "momentum_vision_current.jpg")
            if not os.path.exists(temp_path):
                raise FileNotFoundError("Vision temp file not found.")
                
            img = Image.open(temp_path).convert('RGB')
            width, height = img.size
            
            # Apply foveated cropping if requested
            if request.crop_w is not None and request.crop_h is not None and request.crop_x is not None and request.crop_y is not None:
                x1 = max(0, request.crop_x)
                y1 = max(0, request.crop_y)
                x2 = min(width, x1 + request.crop_w)
                y2 = min(height, y1 + request.crop_h)
                if x2 > x1 and y2 > y1:
                    img = img.crop((x1, y1, x2, y2))
                    
            # Downsize image to prevent llama.cpp segfaults and severely cut down processing time (1 minute -> 5 seconds)
            MAX_DIM = 512
            if img.width > MAX_DIM or img.height > MAX_DIM:
                img.thumbnail((MAX_DIM, MAX_DIM), Image.Resampling.LANCZOS)
                
            img_byte_arr = io.BytesIO()
            img.save(img_byte_arr, format='JPEG', quality=85)
            image_b64 = base64.b64encode(img_byte_arr.getvalue()).decode('utf-8')
            
        except Exception as e:
            import traceback
            traceback.print_exc()
            raise HTTPException(status_code=500, detail=f"Failed to read Vision Image: {e}")
    else:
        raise HTTPException(status_code=400, detail="Fallback base64 not provided. Must use_shm=True")

    # For 8GB hardware optimization, load the 4.5GB model into RAM *only* when requested
    model_path = os.path.join(os.path.dirname(__file__), "momentum-vision-engine.gguf")
    projector_path = os.path.join(os.path.dirname(__file__), "momentum-vision-projector.gguf")
    
    if not os.path.exists(model_path) or not os.path.exists(projector_path):
        raise HTTPException(status_code=500, detail="Vision GGUF models not found on disk.")
        
    try:
        print("Loading Vision Model dynamically into RAM (Hardware Optimization Mode)...", flush=True)
        chat_handler = Llava15ChatHandler(clip_model_path=projector_path)
        temp_llm = Llama(
            model_path=model_path,
            chat_handler=chat_handler,
            n_gpu_layers=-1,
            n_threads=4,
            n_ctx=2048, # Reduced context for RAM
            verbose=False
        )
        
        response = temp_llm.create_chat_completion(
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
        
        # Free the 4.5GB RAM back to the OS!
        result_text = response["choices"][0]["message"]["content"]
        del temp_llm
        del chat_handler
        import gc
        gc.collect()
        
        return {
            "status": "success",
            "text": result_text
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/health")
async def health():
    return {"status": "ok", "model_loaded": llm is not None}

if __name__ == "__main__":
    uvicorn.run("engine:app", host="127.0.0.1", port=8001, reload=True)
