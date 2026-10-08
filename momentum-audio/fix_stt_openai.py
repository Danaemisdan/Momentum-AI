import re

with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace faster-whisper with OpenAI API STT
whisper_init_pattern = r'print\("Initializing Faster-Whisper.*?cpu_threads=4\)'
openai_init = '''import os
    import openai
    import io
    
    api_key = os.environ.get("OPENAI_API_KEY", "sk-dummy")
    print(f"Initializing OpenAI Whisper API STT... (Key loaded: {'Yes' if api_key != 'sk-dummy' else 'No'})")
    app_state["openai_client"] = openai.OpenAI(api_key=api_key)'''
content = re.sub(whisper_init_pattern, openai_init, content, flags=re.DOTALL)

whisper_stt_pattern = r'wav_data = audio\.get_wav_data\(\).*?condition_on_previous_text=False\n\s*\).*?text = " "\.join\(\[segment\.text for segment in segments\]\)\.strip\(\)'
openai_stt = '''wav_data = audio.get_wav_data()
            audio_file = io.BytesIO(wav_data)
            audio_file.name = "audio.wav"
            try:
                transcript = app_state["openai_client"].audio.transcriptions.create(
                    model="whisper-1",
                    file=audio_file
                )
                text = transcript.text.strip()
            except Exception as e:
                print(f"OpenAI STT Error: {e}")
                return'''
content = re.sub(whisper_stt_pattern, openai_stt, content, flags=re.DOTALL)

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
