#!/usr/bin/env bash
# As It Sounds — CLI pipeline test
# Usage: ./test.sh <audio_file> [model]
# Example: ./test.sh .samples/01_19_04.mp3
# Example: ./test.sh .samples/01_19_04.mp3 small

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUDIO="${1:-}"
MODEL="${2:-base}"

if [[ -z "$AUDIO" ]]; then
    echo "Usage: $0 <audio_file> [model=base]"
    echo ""
    echo "Available samples:"
    ls "$SCRIPT_DIR/.samples/" 2>/dev/null | sed 's/^/  .samples\//' || echo "  (no .samples/ directory)"
    exit 1
fi

# Resolve relative path from project root
if [[ "$AUDIO" != /* ]]; then
    AUDIO="$SCRIPT_DIR/$AUDIO"
fi

if [[ ! -f "$AUDIO" ]]; then
    echo "Error: file not found: $AUDIO"
    exit 1
fi

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  As It Sounds — Pipeline Test                           ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo "  파일:  $AUDIO"
echo "  모델:  $MODEL"
echo ""

cargo run \
    --manifest-path "$SCRIPT_DIR/src-tauri/Cargo.toml" \
    --example ais_test \
    --release \
    -- "$AUDIO" "$MODEL"
