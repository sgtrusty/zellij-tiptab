# Variables
PLUGIN_NAME := "zellij-tiptab"
DOCKER_IMAGE := PLUGIN_NAME + "-builder"
DOCKER_EXPORT_DIR := "target/wasm32-wasip1/release"

# Default action: build (via Docker)
default: build

# Show available recipes
help:
    @just --list

# Lint Rust source inside the Docker builder image
lint:
    docker build --target lint -t {{DOCKER_IMAGE}} .

# Compile the Rust code for WASI in release mode (via Docker)
build:
    DOCKER_BUILDKIT=1 docker build --target export \
        --output type=local,dest={{DOCKER_EXPORT_DIR}} \
        -t {{DOCKER_IMAGE}} .
    echo "Extracted plugin to {{DOCKER_EXPORT_DIR}}/{{PLUGIN_NAME}}.wasm"

# Regenerate zellij/Cargo.lock via Docker (and copy it back to the host)
lock:
    DOCKER_BUILDKIT=1 docker build --target builder -t {{DOCKER_IMAGE}} .
    docker create --name {{PLUGIN_NAME}}-lock {{DOCKER_IMAGE}}
    docker cp {{PLUGIN_NAME}}-lock:/app/Cargo.lock Cargo.lock
    docker rm {{PLUGIN_NAME}}-lock

# Build via Docker and launch zellij with the dev layout (dev-docker.template.kdl)
debug: build
    bash -c 'source lib/tiptab-hook.sh && zellij --layout dev-docker.template.kdl'
