import re

with open('server.py', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace OpenAI STT with Google Web Speech API
openai_init = r'from dotenv import load_dotenv.*?app_state\["openai_client"\] = openai\.OpenAI\(api_key=api_key\)'
google_init = '''import speech_recognition as sr

    print("Initializing Google Web Speech API (Free Cloud STT)...")
    r = sr.Recognizer()
    r.pause_threshold = 1.2'''
content = re.sub(openai_init, google_init, content, flags=re.DOTALL)

openai_stt = r'wav_data = audio\.get_wav_data\(\)\s*audio_file = io\.BytesIO\(wav_data\).*?except Exception as e:\s*print\(f"OpenAI STT Error: \{e\}"\)\s*return'
google_stt = '''try:
                # Use Google Web Speech API (Free)
                text = recognizer.recognize_google(audio).strip()
            except sr.UnknownValueError:
                # Silent failure for background noise
                return
            except sr.RequestError as e:
                print(f"STT API Error: {e}")
                return'''
content = re.sub(openai_stt, google_stt, content, flags=re.DOTALL)

with open('server.py', 'w', encoding='utf-8') as f:
    f.write(content)
