#!/bin/bash
cd "$(dirname "$0")"

# Activate the virtual environment
if [ -d "venv" ]; then
    source venv/bin/activate
else
    echo "Virtual environment 'venv' not found. Please run setup first."
    exit 1
fi

echo "Starting Momentum Vision Engine on port 8001..."
python3 engine.py
