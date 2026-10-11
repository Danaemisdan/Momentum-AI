#!/bin/bash
# ===================================================
# Starting Momentum HUD on macOS / Linux
# ===================================================

set -e

# Kill any orphaned processes on our ports
echo "Cleaning up any old processes..."
lsof -ti :44444,8000,8001,1420 2>/dev/null | xargs kill -9 2>/dev/null || true

# Rebuild momentum-agent binary if source has changed or binary is missing
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "⚙️ Updating momentum-agent binary..."
cd "$SCRIPT_DIR/momentum-agent"
cargo build


echo "🚀 Starting Momentum HUD (Tauri Desktop App)..."
cd "$SCRIPT_DIR/momentum-hud"
npx tauri dev
