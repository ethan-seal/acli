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

# Detect container engine (prefer podman, fall back to docker)
# We use a function to handle the different engine configurations
run_container() {
    if command -v podman &> /dev/null; then
        podman "$@"
    elif command -v docker &> /dev/null; then
        if docker info &> /dev/null 2>&1; then
            docker "$@"
        elif sg docker -c "docker info" &> /dev/null 2>&1; then
            # User is in docker group but session hasn't picked it up
            sg docker -c "docker $*"
        else
            sudo docker "$@"
        fi
    else
        echo "Error: Neither podman nor docker found. Please install one of them."
        exit 1
    fi
}

# Detect which engine we're using for display purposes
if command -v podman &> /dev/null; then
    CONTAINER_ENGINE="podman"
elif command -v docker &> /dev/null; then
    if docker info &> /dev/null 2>&1; then
        CONTAINER_ENGINE="docker"
    elif sg docker -c "docker info" &> /dev/null 2>&1; then
        CONTAINER_ENGINE="docker (via sg)"
    else
        CONTAINER_ENGINE="sudo docker"
    fi
else
    echo "Error: Neither podman nor docker found. Please install one of them."
    exit 1
fi
echo "Using container engine: $CONTAINER_ENGINE"

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
IMAGE_EXISTS=false
if command -v podman &> /dev/null; then
    podman image exists acli-anki-e2e && IMAGE_EXISTS=true
else
    run_container image inspect acli-anki-e2e &> /dev/null && IMAGE_EXISTS=true
fi

if [ "$REBUILD_CONTAINER" = true ] || [ "$IMAGE_EXISTS" = false ]; then
    echo "Building demo container..."
    ./e2e/build.sh
fi

# Clean and create output directory structure
# Files from previous runs may be owned by container user, so use container engine to remove them
if [ -d "$OUTPUT_DIR" ]; then
    echo "Cleaning previous output..."
    if command -v podman &> /dev/null; then
        podman unshare rm -rf "$OUTPUT_DIR" 2>/dev/null || rm -rf "$OUTPUT_DIR" 2>/dev/null || true
    else
        rm -rf "$OUTPUT_DIR" 2>/dev/null || sudo rm -rf "$OUTPUT_DIR" 2>/dev/null || true
    fi
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

run_container run --rm \
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
