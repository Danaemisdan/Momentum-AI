import re
with open('engine.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace the complicated SHM logic with a simple file reader
old_shm_logic = """    if request.use_shm:
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
                        shm = shared_memory.SharedMemory(name=f"Local\\\\{real_name}")
                    except FileNotFoundError:
                        shm = shared_memory.SharedMemory(name=f"Global\\\\{real_name}")
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
            raise HTTPException(status_code=500, detail=f"Failed to read from Shared Memory: {e}")"""

new_file_logic = """    if request.use_shm:
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
            raise HTTPException(status_code=500, detail=f"Failed to read Vision Image: {e}")"""

content = content.replace(old_shm_logic, new_file_logic)

with open('engine.py', 'w', encoding='utf-8') as f:
    f.write(content)
