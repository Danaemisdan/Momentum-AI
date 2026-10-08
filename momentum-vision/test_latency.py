import asyncio, websockets, time, json
async def test():
    t=time.time()
    async with websockets.connect('ws://127.0.0.1:44444/chat') as ws:
        await ws.send(json.dumps({'history':[], 'message': 'what is the capital of india?'}))
        print('Sent. Waiting...')
        msg = await ws.recv()
        print(f'Got {msg} in {time.time()-t:.2f}s')
asyncio.run(test())
