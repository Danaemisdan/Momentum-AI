import re
with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace('def stt_worker():\n        recognizer', 'def stt_worker():\n        import traceback\n        try:\n            recognizer')
content = content.replace('import threading', '        except Exception as e:\n            print(f"STT ERROR: {e}", flush=True)\n            traceback.print_exc()\n    import threading')

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
