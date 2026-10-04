import asyncio
import websockets

async def test():
    try:
        async with websockets.connect("ws://127.0.0.1:8000/stt") as ws:
            print("Connected to /stt!")
            await asyncio.sleep(2)
    except Exception as e:
        print(f"Error: {e}")

asyncio.run(test())
