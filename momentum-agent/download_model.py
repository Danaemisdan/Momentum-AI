import urllib.request
import time
import os

url = "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf"
file_path = "momentum-engine-0.5b.gguf"

max_retries = 10
retries = 0

while retries < max_retries:
    try:
        req = urllib.request.Request(url)
        if os.path.exists(file_path):
            downloaded = os.path.getsize(file_path)
            req.add_header("Range", f"bytes={downloaded}-")
        else:
            downloaded = 0

        with urllib.request.urlopen(req, timeout=10) as response:
            total_length = int(response.headers.get('content-length', 0)) + downloaded
            
            mode = "ab" if downloaded > 0 else "wb"
            with open(file_path, mode) as f:
                while True:
                    chunk = response.read(8192 * 4)
                    if not chunk:
                        break
                    f.write(chunk)
                    downloaded += len(chunk)
                    print(f"Downloaded {downloaded / 1024 / 1024:.2f} MB / {total_length / 1024 / 1024:.2f} MB", end='\r')
                    
        if downloaded >= total_length:
            print("\nDownload complete!")
            break
            
    except Exception as e:
        print(f"\nError: {e}")
        retries += 1
        time.sleep(2)
        print("Retrying...")

