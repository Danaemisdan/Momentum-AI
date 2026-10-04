import base64
import requests
import json

# Create a 1x1 black pixel GIF in base64
dummy_img = "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7"

payload = {
    "image_base64": dummy_img,
    "prompt": "What color is this image?",
    "max_tokens": 50
}

try:
    print("Sending request to Vision Engine on port 8001...")
    res = requests.post("http://127.0.0.1:8001/perceive", json=payload, timeout=60)
    print(f"Status Code: {res.status_code}")
    print(f"Response: {json.dumps(res.json(), indent=2)}")
except Exception as e:
    print(f"Connection failed: {e}")
