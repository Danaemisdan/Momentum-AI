import re

with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Add load_dotenv support
dotenv_init = '''import os
    import openai
    import io
    from dotenv import load_dotenv
    load_dotenv(os.path.join(os.path.dirname(os.path.dirname(__file__)), '.env'))
    
    api_key = os.environ.get("OPENAI_API_KEY", "sk-dummy")
    print(f"Initializing OpenAI Whisper API STT... (Key loaded: {'Yes' if api_key != 'sk-dummy' else 'No'})")
    app_state["openai_client"] = openai.OpenAI(api_key=api_key)'''

content = re.sub(r'import os\s+import openai\s+import io\s+api_key = os\.environ\.get\("OPENAI_API_KEY", "sk-dummy"\).*?api_key=api_key\)', dotenv_init, content, flags=re.DOTALL)

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
