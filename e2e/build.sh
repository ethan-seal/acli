#!/usr/bin/env bash
# Build the Anki e2e testing container
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Building Anki e2e container..."
echo "This may take a while on first build (compiling Anki from source)."
echo

podman build \
    -t acli-anki-e2e \
    -f e2e/Containerfile \
    --format docker \
    .

echo
echo "Build complete! Run with: ./e2e/run.sh"
