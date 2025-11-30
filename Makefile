# Default: build for current architecture
desktop-icons: desktop-icons.rs Cargo.toml
	cargo build --release
	cp target/release/desktop-icons desktop-icons

# Cross-compile for Raspberry Pi 4 (64-bit ARM) using Podman
pi4: docker-build.sh
	@chmod +x docker-build.sh
	@./docker-build.sh

# Alternative: try native cross-compile (may not work on macOS)
pi4-native: desktop-icons.rs Cargo.toml
	@echo "WARNING: Native cross-compile may fail on macOS due to missing ARM GTK libraries"
	@echo "Use 'make pi4' for Podman-based build instead"
	@rustup target add aarch64-unknown-linux-gnu
	cargo build --release --target aarch64-unknown-linux-gnu
	@if [ -f target/aarch64-unknown-linux-gnu/release/desktop-icons ]; then \
		cp target/aarch64-unknown-linux-gnu/release/desktop-icons desktop-icons && \
		echo "Binary created: desktop-icons"; \
	else \
		echo "ERROR: Build failed - binary not found"; \
		exit 1; \
	fi

check-deps:
	@echo "Checking build dependencies..."
	@command -v rustc >/dev/null 2>&1 || { echo "ERROR: rustc not found. Install with: sudo apt install rustc"; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo not found. Install with: sudo apt install cargo"; exit 1; }
	@pkg-config --exists gtk+-3.0 2>/dev/null || { echo "ERROR: GTK3 development packages not found. Install with: sudo apt install libgtk-3-dev pkg-config libcairo2-dev libpango1.0-dev"; exit 1; }
	@echo "Build dependencies OK"
	@echo ""
	@echo "Checking runtime dependencies..."
	@command -v yad >/dev/null 2>&1 || { echo "WARNING: yad not found (needed for home button). Install with: sudo apt install yad"; }
	@command -v jq >/dev/null 2>&1 || { echo "WARNING: jq not found (needed for home button). Install with: sudo apt install jq"; }
	@echo "Runtime dependencies OK"

install: desktop-icons
	cp desktop-icons /usr/local/bin/ || cp desktop-icons ~/.local/bin/

clean:
	cargo clean
	rm -f desktop-icons

.PHONY: install clean check-deps pi4 pi4-native

