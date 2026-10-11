# Momentum AI

> **A voice-native, vision-enabled autonomous operating system companion with superhuman digital agency.**  
> Built for macOS (Apple Silicon Metal) and Windows.

Momentum is a lightning-fast, highly conversational autonomous AI agent. It features ultra-low latency speech-to-text (STT), human-like text-to-speech (TTS), deep system-level awareness, a real-time vision engine, and an autonomous UI operator that drives applications across desktop and web.

---

## 📚 Core Architecture & Documentation

* **[MOMENTUM_OPERATIVE_BIBLE.md](file:///Users/sanjeevn/Downloads/Momentum%20AI/MOMENTUM_OPERATIVE_BIBLE.md)**: Master codex detailing cross-industry capabilities (3D Modeling, Photoshop/Design, Game Dev, Web-to-Web Creation, AI Apps, Client Acquisition, Daily OS Housekeeping), Virtual Call Telepresence (camera avatar & mic loopback), 30 FPS Spatial Vision, and dynamic UI agency.
* **[MOMENTUM_SPEC.md](file:///Users/sanjeevn/Downloads/Momentum%20AI/MOMENTUM_SPEC.md)**: Full technical architecture specification, 7-phase implementation roadmap, sensory checks, Bézier mouse physics, and the Self-Evolving Knowledge Graph.

---

## Setup Instructions

Because Momentum relies on several local machine learning models that are too large to host on GitHub (>100MB), you will need to manually download and place a few dependencies before starting the system.

### 1. Download Model Weights
Please download the following models and place them in their respective directories:

**Audio & Voice Models (`momentum-audio/`)**
* `kokoro-v1.0.onnx` -> Place inside `momentum-audio/`
* `base_voice.onnx` -> Place inside `momentum-audio/models/`
* `vc_model.pth` -> Place inside `momentum-audio/models/`

**Vision Models (`momentum-vision/`)**
* `momentum-vision-engine.gguf` -> Place inside `momentum-vision/`
* `momentum-vision-projector.gguf` -> Place inside `momentum-vision/`

### 2. Configure OpenRouter API Key
To protect your account, API keys are not hardcoded in the repository. Set your `OPENROUTER_API_KEY` in your system environment variables, or configure it directly in your environment.

### 3. Install Dependencies
Make sure you have Node.js, Python 3.10+, and Rust installed. 

Install the frontend dependencies:
```bash
cd momentum-hud
npm install
```

Install the backend Python dependencies:
```bash
cd momentum-audio
pip install -r requirements.txt
```

### 4. Run the System
You can boot the entire suite at once by running the following command from the root directory:
```bash
./run_momentum.sh
```
Or manually run:
```bash
killall momentum_hud 2>/dev/null; pgrep -f "tauri" | xargs kill -9 2>/dev/null; pgrep -f "uvicorn" | xargs kill -9 2>/dev/null; pgrep -f "momentum-agent" | xargs kill -9 2>/dev/null; lsof -ti :1420,8000,8001 | xargs kill -9 2>/dev/null; cd momentum-agent && cargo build && cd ../momentum-audio && nohup uvicorn server:app --port 8000 > audio.log 2>&1 & cd ../momentum-agent && nohup cargo run > agent.log 2>&1 & cd ../momentum-hud && nohup npx tauri dev > tauri.log 2>&1 & sleep 3
```

Enjoy your lightning-fast AI companion!
