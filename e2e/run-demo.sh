#!/usr/bin/env bash
# Run the acli demo in the container and generate a report
#
# This script:
# 1. Builds acli with real-anki feature
# 2. Builds the demo container if needed
# 3. Runs the demo inside the container (TypeScript/Bun)
# 4. Generates an HTML report with screenshots
#
# Usage:
#   ./e2e/run-demo.sh                     # Run the demo
#   ./e2e/run-demo.sh --rebuild-container # Rebuild the container first

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_ROOT/e2e/demo-output"

# Parse arguments
REBUILD_CONTAINER=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --rebuild-container)
            REBUILD_CONTAINER=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--rebuild-container]"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

# Always build acli with real-anki feature to ensure it can write to Anki collections
echo "Building acli with real-anki feature..."
cd anki-wrapper
cargo build --release --features real-anki -p acli --manifest-path ../acli/Cargo.toml
cd "$PROJECT_ROOT"

ACLI_BINARY="$PROJECT_ROOT/target/release/acli"

# Check if container needs building
if [ "$REBUILD_CONTAINER" = true ] || ! podman image exists acli-anki-e2e; then
    echo "Building demo container..."
    ./e2e/build.sh
fi

# Clean and create output directory structure
# Files from previous runs may be owned by container user, so use podman to remove them
if [ -d "$OUTPUT_DIR" ]; then
    echo "Cleaning previous output..."
    podman unshare rm -rf "$OUTPUT_DIR" 2>/dev/null || rm -rf "$OUTPUT_DIR" 2>/dev/null || true
fi
mkdir -p "$OUTPUT_DIR/screenshots"
mkdir -p "$OUTPUT_DIR/content"
mkdir -p "$OUTPUT_DIR/anki_collection"
chmod -R 777 "$OUTPUT_DIR"

# Patch the binary for container compatibility (NixOS builds have incompatible interpreter)
# Must happen AFTER cleaning output directory since patched binary is stored there
ACLI_PATCHED="$OUTPUT_DIR/acli-patched"

if readelf -l "$ACLI_BINARY" 2>/dev/null | grep -q '/nix/store'; then
    echo "Patching binary for container compatibility..."
    if command -v patchelf &> /dev/null; then
        cp "$ACLI_BINARY" "$ACLI_PATCHED"
        patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 "$ACLI_PATCHED"
        ACLI_BINARY="$ACLI_PATCHED"
    elif command -v nix-shell &> /dev/null; then
        cp "$ACLI_BINARY" "$ACLI_PATCHED"
        nix-shell -p patchelf --run "patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 '$ACLI_PATCHED'"
        ACLI_BINARY="$ACLI_PATCHED"
    else
        echo "WARNING: Binary may have incompatible interpreter. Install patchelf to fix."
    fi
fi

echo "Using acli binary: $ACLI_BINARY"

# Run the demo in the container using Bun/TypeScript
echo
echo "Running demo in container (Bun/TypeScript)..."
echo "============================================================"

podman run --rm \
    --name acli-demo \
    -v "$PROJECT_ROOT:/project:ro" \
    -v "$ACLI_BINARY:/home/anki/acli:ro" \
    -v "$OUTPUT_DIR:/home/anki/output" \
    -v "$SCRIPT_DIR/demo:/home/anki/demo:ro" \
    acli-anki-e2e \
    bun run /home/anki/demo/run-demo.ts \
        --acli-binary /home/anki/acli \
        --output-dir /home/anki/output

echo
echo "============================================================"
echo "Demo complete!"
echo
echo "Report: $OUTPUT_DIR/demo_report.html"
echo
echo "Open it with:"
echo "  xdg-open $OUTPUT_DIR/demo_report.html"
