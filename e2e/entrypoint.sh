#!/bin/bash
# Entrypoint script for Anki e2e testing container
set -e

# Create XDG runtime directory if needed
if [ ! -d "$XDG_RUNTIME_DIR" ]; then
    mkdir -p "$XDG_RUNTIME_DIR"
    chmod 700 "$XDG_RUNTIME_DIR"
fi

# If running headless (no DISPLAY), start Xvfb and window manager
if [ -z "$DISPLAY" ]; then
    echo "No DISPLAY set, starting Xvfb..."
    export DISPLAY=:99
    Xvfb :99 -screen 0 1280x1024x24 &
    sleep 1
    
    # Start openbox window manager for proper window management
    openbox &
    sleep 1
fi

# Execute the command
exec "$@"
