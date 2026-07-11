# Variables
PLUGIN_NAME=zellij-tiptab
SOURCE_FILE=target/wasm32-wasip1/release/$(PLUGIN_NAME).wasm
DEST_DIR=$(HOME)/.config/zellij/plugins
DEST_FILE=$(DEST_DIR)/$(PLUGIN_NAME).wasm

.PHONY: help build clean install uninstall

# `make` with no target shows this help
help:
	@echo "Available targets:"
	@echo "  make build         Build the plugin with cargo (wasm32-wasip1)"
	@echo "  make clean         Remove build artifacts"
	@echo "  make install       Install plugin and bash hook (interactive)"
	@echo "  make uninstall     Uninstall plugin and bash hook"

# Compile the Rust code for WASI in release mode (via cargo)
build:
	cargo build --release --target wasm32-wasip1

# Clean the build artifacts
clean:
	cargo clean

# Install the plugin to the local zellij plugins dir
install:
	@read -p "Plugin install path [$(DEST_FILE)]: " plugin_path && \
	plugin_path="$${plugin_path:-$(DEST_FILE)}" && \
	mkdir -p "$$(dirname "$$plugin_path")" && \
	cp $(SOURCE_FILE) "$$plugin_path" && \
	echo "Installed plugin to: $$plugin_path" && \
	echo "Zellij KDL path: file:$$plugin_path"
	@read -p "Bash hook install path [~/.bashrc.d/16-external]: " hook_path && \
	hook_path="$${hook_path:-$$HOME/.bashrc.d/16-external}" && \
	mkdir -p "$$(dirname "$$hook_path")" && \
	cp lib/tiptab-hook.sh "$$hook_path" && \
	echo "Installed bash hook to: $$hook_path"

# Uninstall the plugin and bash hook
uninstall:
	rm -f $(DEST_FILE)
	rm -f $$HOME/.bashrc.d/16-external
	@echo "Uninstalled plugin and bash hook"
