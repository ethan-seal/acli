#!/usr/bin/env bash
# Run the acli demo in the container and generate a report
#
# This script:
# 1. Optionally builds acli with real-anki feature
# 2. Builds the demo container if needed
# 3. Runs the demo inside the container (TypeScript/Bun)
# 4. Copies the report to the host
#
# Usage:
#   ./e2e/run-demo.sh                    # Use existing acli binary
#   ./e2e/run-demo.sh --build-acli       # Build acli first (slow!)
#   ./e2e/run-demo.sh --rebuild-container # Rebuild the container

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_ROOT/e2e/demo-output"

# Parse arguments
BUILD_ACLI=false
REBUILD_CONTAINER=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --build-acli)
            BUILD_ACLI=true
            shift
            ;;
        --rebuild-container)
            REBUILD_CONTAINER=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--build-acli] [--rebuild-container]"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

# Build acli if requested
ACLI_BINARY=""
if [ "$BUILD_ACLI" = true ]; then
    echo "Building acli with real-anki feature..."
    echo "WARNING: This requires the full Anki build and may take 30+ minutes."
    echo
    
    # Build in the anki-wrapper workspace
    cd anki-wrapper
    cargo build --release --features real-anki -p acli --manifest-path ../acli/Cargo.toml
    cd "$PROJECT_ROOT"
    
    ACLI_BINARY="$PROJECT_ROOT/anki-wrapper/target/release/acli"
else
    # Look for existing binary
    if [ -f "$PROJECT_ROOT/target/release/acli" ]; then
        ACLI_BINARY="$PROJECT_ROOT/target/release/acli"
    elif [ -f "$PROJECT_ROOT/anki-wrapper/target/release/acli" ]; then
        ACLI_BINARY="$PROJECT_ROOT/anki-wrapper/target/release/acli"
    else
        echo "ERROR: acli binary not found."
        echo "Either build it with --build-acli or ensure it exists at:"
        echo "  $PROJECT_ROOT/target/release/acli"
        echo "  $PROJECT_ROOT/anki-wrapper/target/release/acli"
        exit 1
    fi
fi

echo "Using acli binary: $ACLI_BINARY"

# Patch the binary for container compatibility (NixOS builds have incompatible interpreter)
ACLI_PATCHED="$PROJECT_ROOT/e2e/demo-output/acli-patched"
mkdir -p "$(dirname "$ACLI_PATCHED")"

# Check if we need to patch (NixOS binaries have /nix/store interpreter)
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

# Check if container needs building
if [ "$REBUILD_CONTAINER" = true ] || ! podman image exists acli-anki-e2e; then
    echo "Building demo container..."
    ./e2e/build.sh
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

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
