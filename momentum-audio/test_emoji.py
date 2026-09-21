import asyncio
from kokoro_onnx import Kokoro

async def run():
    kokoro = Kokoro("kokoro-v1.0.onnx", "voices-v1.0.bin")
    try:
        samples, rate = kokoro.create("🗣️ Momentum: AI automations, huh?", voice="af_heart", speed=1.05, lang="en-us")
        print(f"Success! {len(samples)} samples generated.")
    except Exception as e:
        print(f"Error: {e}")

asyncio.run(run())
