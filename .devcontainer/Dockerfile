FROM debian:trixie

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    clang \
    lldb \
    cmake \
    ca-certificates \
    sqlite3 \
    libsqlite3-dev \
    bash \
    && rm -rf /var/lib/apt/lists/*

# Install rustup + latest stable Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --default-toolchain stable \
    && rustup component add rustfmt clippy rust-src

# Create non-root user (recommended)
RUN useradd -ms /bin/bash dev
USER dev
WORKDIR /workspace