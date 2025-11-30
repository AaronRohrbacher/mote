FROM rust:1.82-slim

# Install cross-compilation toolchain and ARM64 libraries
RUN dpkg --add-architecture arm64 && \
    apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    pkg-config \
    libgtk-3-dev:arm64 \
    libcairo2-dev:arm64 \
    libpango1.0-dev:arm64 \
    libgdk-pixbuf2.0-dev:arm64 \
    libglib2.0-dev:arm64 \
    && rm -rf /var/lib/apt/lists/*

# Install Rust target for 64-bit ARM
RUN rustup target add aarch64-unknown-linux-gnu

WORKDIR /build

COPY Cargo.toml desktop-icons.rs ./

# Set up cross-compilation environment for ARM64
ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH_aarch64_unknown_linux_gnu=/usr/lib/aarch64-linux-gnu/pkgconfig
ENV PKG_CONFIG_SYSROOT_DIR_aarch64_unknown_linux_gnu=/

# Build for 64-bit ARM
RUN cargo build --release --target aarch64-unknown-linux-gnu

