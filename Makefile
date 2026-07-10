# Variables
PLUGIN_NAME=zellij-tiptab
SOURCE_FILE=target/wasm32-wasip1/release/$(PLUGIN_NAME).wasm
DEST_DIR=$(HOME)/.config/zellij/plugins
DEST_FILE=$(DEST_DIR)/$(PLUGIN_NAME).wasm
DOCKER_IMAGE=$(PLUGIN_NAME)-builder
DOCKER_EXPORT_DIR=target/wasm32-wasip1/release

.PHONY: help build docker-build clean install debug

# `make` with no target shows this help
help:
	@echo "Available targets:"
	@echo "  make build         Build the plugin with cargo (wasm32-wasip1)"
	@echo "  make docker-build  Build the plugin via Docker"
	@echo "  make debug         Build via Docker and launch zellij with the dev layout"
	@echo "  make clean         Remove build artifacts"
	@echo "  make install       Install the built plugin to $(DEST_DIR)"

# Compile the Rust code for WASI in release mode (via cargo)
build:
	cargo build --release --target wasm32-wasip1

# Build the plugin via Docker and extract the .wasm
docker-build:
	DOCKER_BUILDKIT=1 docker build --target export \
		--output type=local,dest=$(DOCKER_EXPORT_DIR) \
		-t $(DOCKER_IMAGE) .
	@echo "Extracted plugin to $(DOCKER_EXPORT_DIR)/$(PLUGIN_NAME).wasm"

# Clean the build artifacts
clean:
	cargo clean

# Build via Docker and launch zellij with the dev layout (dev-docker.template.kdl)
debug: docker-build
	bash -c 'source lib/tiptab-hook.sh && zellij --layout dev-docker.template.kdl'

# Install the plugin to the local zellij plugins dir
install:
	mkdir -p $(DEST_DIR)
	cp $(SOURCE_FILE) $(DEST_FILE)
	@echo "------------------------------------------------"
	@echo "Successfully installed to: $(DEST_FILE)"
	@echo "Zellij KDL path: file:$(DEST_FILE)"
	@echo "------------------------------------------------"
