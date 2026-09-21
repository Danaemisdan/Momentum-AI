#!/bin/bash

# ==========================================================
# MOMENTUM AUDIO MICROSERVICE - PIPER + RVC SETUP
# ==========================================================

AUDIO_DIR="/Users/sanjeevn/Downloads/Momentum AI/momentum-audio"
MODEL_DIR="$AUDIO_DIR/models"

echo "⚡ Booting Cross-Platform Optimization Matrix..."

cd "$AUDIO_DIR" || exit
mkdir -p "$MODEL_DIR"

# 1. Initialize Python Native Virtual Environment
echo "🐍 Creating Virtual Environment..."
if [ ! -d "venv" ]; then
    python3 -m venv venv
fi

# Activate cleanly
source venv/bin/activate

# 2. Deploy Dependencies (CPU FP32 Optimized explicitly for Cross-Platform 8GB scaling without massive CUDA failures)
echo "📦 Installing Engine Libraries (Torch, Piper, RVC, FastAPI)..."
pip install fastapi "uvicorn[standard]" websockets numpy soundfile pedalboard
pip install piper-tts
# The `rvc-python` port handles native Fairseq HubERT embedding conversion specifically locally!
pip install rvc-python torch torchaudio

# 3. Download the Piper Base Generation Neural Weights
# We utilize the "lessac-high" standard specifically because it breathes and paces phenomenally well.
if [ ! -f "$MODEL_DIR/base_voice.onnx" ]; then
    echo "⬇️ Downloading Piper Fast-Inference Base Model..."
    curl -L -o "$MODEL_DIR/base_voice.onnx" "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/high/en_US-lessac-high.onnx"
    curl -L -o "$MODEL_DIR/base_voice.onnx.json" "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/high/en_US-lessac-high.onnx.json"
fi

echo ""
echo "✅ PIPELINE PREPARED SUCCESSFULLY!"
echo "⚠️ IMPORTANT: You must manually drop your RVC '.pth' and '.index' files into the folder:"
echo "   /Users/sanjeevn/Downloads/Momentum AI/momentum-audio/models"
echo "   Rename them strictly to: 'vc_model.pth' and 'vc_model.index'"
echo ""
echo "🚀 To Run the Server, simply execute:"
echo "   source \"/Users/sanjeevn/Downloads/Momentum AI/momentum-audio/venv/bin/activate\""
echo "   uvicorn server:app --host 127.0.0.1 --port 8000"
