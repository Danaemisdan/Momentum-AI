import asyncio
import websockets

async def run():
    async with websockets.connect("ws://127.0.0.1:8000/tts") as ws:
        chunks = [
            "🗣️ Momentum: AI automations, huh?", 
            "That's a pretty cool niche.", 
            "I've got a few connections in the industry, but I need a bit more info to give you some decent leads.", 
            "What specific area of AI automations are you looking to break into?", 
            "Chatbots, workflow automation, or something else?", 
            "And by the way, I've got to say, it's not every day I get to talk to someone who's actually interested in AI automations."
        ]
        
        for c in chunks:
            print(f"Sending: {c}")
            await ws.send(c)
            
        audio_buffers = []
        # Receive them all
        for i in range(len(chunks)):
            try:
                res = await asyncio.wait_for(ws.recv(), timeout=20.0)
                print(f"Received audio back for chunk {i}, length {len(res)}")
                audio_buffers.append(res)
            except Exception as e:
                print(f"Failed receiving chunk {i}: {e}")
                
        with open("output_dump.wav", "wb") as f:
            for b in audio_buffers:
                f.write(b)
                
asyncio.run(run())
