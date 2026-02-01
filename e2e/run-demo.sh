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
    # Look for existing binary - prefer anki-wrapper binary (has real-anki feature)
    if [ -f "$PROJECT_ROOT/anki-wrapper/target/release/acli" ]; then
        ACLI_BINARY="$PROJECT_ROOT/anki-wrapper/target/release/acli"
    elif [ -f "$PROJECT_ROOT/target/release/acli" ]; then
        ACLI_BINARY="$PROJECT_ROOT/target/release/acli"
    else
        echo "ERROR: acli binary not found."
        echo "Either build it with --build-acli or ensure it exists at:"
        echo "  $PROJECT_ROOT/anki-wrapper/target/release/acli"
        echo "  $PROJECT_ROOT/target/release/acli"
        exit 1
    fi
fi

# Verify the binary has real-anki feature by testing if it creates a collection
echo "Verifying acli binary has real-anki feature..."
TEST_DIR=$(mktemp -d)
mkdir -p "$TEST_DIR/content" "$TEST_DIR/collection"
echo '- Test
  - Hello <-> World' > "$TEST_DIR/content/test.md"
"$ACLI_BINARY" sync --source "$TEST_DIR/content" --deck Test --collection "$TEST_DIR/collection/collection.anki2" >/dev/null 2>&1

if [ ! -f "$TEST_DIR/collection/collection.anki2" ]; then
    rm -rf "$TEST_DIR"
    echo
    echo "ERROR: The acli binary does not have the 'real-anki' feature enabled."
    echo "It cannot write to actual Anki collections."
    echo
    echo "To fix this, run with --build-acli to build with the real-anki feature:"
    echo "  ./e2e/run-demo.sh --build-acli"
    echo
    echo "Note: First build takes 30+ minutes to compile the Anki library."
    exit 1
fi
rm -rf "$TEST_DIR"
echo "Binary verified: real-anki feature is enabled."

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
