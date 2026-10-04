#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUDIO_DIR="$ROOT_DIR/momentum-audio"
AGENT_DIR="$ROOT_DIR/momentum-agent"
UI_DIR="$ROOT_DIR/ui"

AUDIO_PID=""
AGENT_PID=""
UI_PID=""

cleanup() {
  local exit_code=$?
  trap - EXIT INT TERM

  for pid in "$UI_PID" "$AGENT_PID" "$AUDIO_PID"; do
    if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" 2>/dev/null || true
    fi
  done

  wait || true
  exit "$exit_code"
}

trap cleanup EXIT INT TERM

require_dir() {
  local path=$1
  local label=$2
  if [[ ! -d "$path" ]]; then
    echo "Missing $label at: $path" >&2
    exit 1
  fi
}

require_file() {
  local path=$1
  local label=$2
  if [[ ! -f "$path" ]]; then
    echo "Missing $label at: $path" >&2
    exit 1
  fi
}

require_dir "$AUDIO_DIR" "audio service directory"
require_dir "$AGENT_DIR" "agent directory"
require_dir "$UI_DIR" "UI directory"
require_dir "$AUDIO_DIR/venv" "audio virtualenv"
require_file "$AGENT_DIR/Cargo.toml" "agent Cargo.toml"
require_file "$UI_DIR/package.json" "UI package.json"

wait_for_http() {
  local url=$1
  local label=$2
  local attempts=${3:-60}

  for ((i = 1; i <= attempts; i++)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      echo "$label is ready at $url"
      return 0
    fi
    sleep 1
  done

  echo "$label failed to become ready at $url" >&2
  return 1
}

echo "Starting Momentum audio on http://127.0.0.1:8000"
(
  cd "$AUDIO_DIR"
  source venv/bin/activate
  exec uvicorn server:app --host 127.0.0.1 --port 8000
) &
AUDIO_PID=$!

echo "Starting Momentum agent on http://127.0.0.1:3000"
(
  cd "$AGENT_DIR"
  exec cargo run
) &
AGENT_PID=$!

echo "Starting Momentum UI on http://127.0.0.1:3001"
(
  cd "$UI_DIR"
  npm run build
  exec npm run start
) &
UI_PID=$!

echo ""
echo "Momentum is booting."
echo "UI:    http://localhost:3001"
echo "Agent: ws://127.0.0.1:3000/chat"
echo "Audio: ws://127.0.0.1:8000/tts"
echo ""
echo "Press Ctrl+C to stop everything."

wait_for_http "http://127.0.0.1:8000/health" "Audio service"
wait_for_http "http://127.0.0.1:3000/health" "Agent service"
wait_for_http "http://127.0.0.1:3001/" "UI"

wait
