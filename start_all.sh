#!/bin/bash
echo "Cleaning up any old processes..."
kill -9 $(pgrep -f momentum-agent) 2>/dev/null
kill -9 $(pgrep -f 'next dev') 2>/dev/null
kill -9 $(pgrep -f 'uvicorn server:app') 2>/dev/null
kill -9 $(pgrep -f 'python engine.py') 2>/dev/null
kill -9 $(pgrep -f 'engine.py') 2>/dev/null
lsof -ti :8000,8001,44444,44445 | xargs kill -9 2>/dev/null
# Trap SIGINT (Ctrl+C) and SIGTERM to kill all background child processes
trap 'echo "\nShutting down Momentum AI..."; kill -9 $AGENT_PID $AUDIO_PID $VISION_PID $UI_PID 2>/dev/null; exit' INT TERM EXIT

echo "Starting Momentum Agent on port 44444..."
cd "/Users/sanjeevn/Downloads/Momentum AI/momentum-agent"
cargo run > agent.log 2>&1 &
AGENT_PID=$!

echo "Starting Momentum Audio on port 8000..."
cd "/Users/sanjeevn/Downloads/Momentum AI/momentum-audio"
if [ -d "venv" ]; then
    source venv/bin/activate
fi
uvicorn server:app --host 127.0.0.1 --port 8000 > audio.log 2>&1 &
AUDIO_PID=$!

echo "Starting Momentum Vision on port 8001..."
cd "/Users/sanjeevn/Downloads/Momentum AI/momentum-vision"
if [ -d "venv" ]; then
    source venv/bin/activate
fi
python engine.py > vision.log 2>&1 &
VISION_PID=$!

echo "Starting Next.js UI on port 44445..."
cd "/Users/sanjeevn/Downloads/Momentum AI/ui"
npm run dev > ui.log 2>&1 &
UI_PID=$!

echo "All 4 core systems are running!"
echo "Agent PID: $AGENT_PID"
echo "Audio PID: $AUDIO_PID"
echo "Vision PID: $VISION_PID"
echo "UI PID: $UI_PID"
echo "The UI is accessible at: http://localhost:44445"
wait
