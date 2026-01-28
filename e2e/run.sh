#!/usr/bin/env bash
# Run the Anki e2e testing container
#
# Usage:
#   ./e2e/run.sh                    # Run Anki GUI (requires X11)
#   ./e2e/run.sh --headless         # Run with virtual framebuffer
#   ./e2e/run.sh bash               # Get a shell in the container
#   ./e2e/run.sh --headless python  # Run Python in headless mode
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default options
HEADLESS=false
INTERACTIVE=true
CONTAINER_ARGS=()
CMD_ARGS=()

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --headless|-H)
            HEADLESS=true
            shift
            ;;
        --no-interactive|-n)
            INTERACTIVE=false
            shift
            ;;
        *)
            CMD_ARGS+=("$1")
            shift
            ;;
    esac
done

# Build common container args
CONTAINER_ARGS+=(
    --rm
    --name acli-anki-e2e
    # Mount project for development
    -v "$PROJECT_ROOT:/project:ro"
    # Persistent Anki data for testing
    -v "$PROJECT_ROOT/e2e/data:/home/anki/.local/share/Anki2"
    # Test collections directory
    -v "$PROJECT_ROOT/e2e/collections:/home/anki/collections"
)

# Add interactive flags
if [ "$INTERACTIVE" = true ]; then
    CONTAINER_ARGS+=(-it)
fi

# Set up X11 or headless mode
if [ "$HEADLESS" = true ]; then
    echo "Running in headless mode (Xvfb)"
    # No DISPLAY set - entrypoint will start Xvfb
else
    # X11 forwarding
    if [ -n "$DISPLAY" ]; then
        echo "Using host X11 display: $DISPLAY"
        CONTAINER_ARGS+=(
            -e DISPLAY="$DISPLAY"
            -v /tmp/.X11-unix:/tmp/.X11-unix:ro
        )
        
        # Allow container to connect to X server
        xhost +local: 2>/dev/null || true
    else
        echo "Warning: No DISPLAY set. Use --headless for virtual framebuffer."
    fi
fi

# Create data directories if they don't exist
mkdir -p "$PROJECT_ROOT/e2e/data"
mkdir -p "$PROJECT_ROOT/e2e/collections"

# Run container
echo "Starting container..."
podman run "${CONTAINER_ARGS[@]}" acli-anki-e2e "${CMD_ARGS[@]}"
