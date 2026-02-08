#!/usr/bin/env bash
# Build the Anki e2e testing container
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Function to run container commands
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

echo "Building Anki e2e container..."
echo "This may take a while on first build (compiling Anki from source)."
echo

BUILD_ARGS="-t acli-anki-e2e -f e2e/Containerfile --network host"
if command -v podman &> /dev/null; then
    BUILD_ARGS="$BUILD_ARGS --format docker"
fi

run_container build $BUILD_ARGS .

echo
echo "Build complete! Run with: ./e2e/run.sh"
