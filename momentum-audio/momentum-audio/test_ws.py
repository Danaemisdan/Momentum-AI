import asyncio
import websockets

async def test():
    try:
        async with websockets.connect("ws://127.0.0.1:8000/tts") as ws:
            print("Connected to TTS!")
            await ws.send("Testing TTS over WebSocket.")
            print("Sent message.")
            await asyncio.sleep(2)
            print("Done.")
    except Exception as e:
        print(f"Error: {e}")

asyncio.run(test())
