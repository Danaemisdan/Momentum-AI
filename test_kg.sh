#!/usr/bin/env bash
set -e

# Change directory to momentum-agent
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/momentum-agent"

echo "🧪 Running Momentum AI Knowledge Graph & Autonomous Researcher Harness..."
if [ "$#" -gt 0 ]; then
    cargo run --bin test_kg -- "$@"
else
    cargo run --bin test_kg
fi
