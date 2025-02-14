import asyncio
import websockets

async def test_websocket():
    uri = "ws://127.0.0.1:8008"
    async with websockets.connect(uri) as websocket:
        # Send a message
        await websocket.send("Hello WebSocket!")
        print("Sent: Hello WebSocket!")
        
        # Receive the echo response
        response = await websocket.recv()
        print(f"Received: {response}")

asyncio.run(test_websocket())