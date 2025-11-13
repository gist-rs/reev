# syntax=docker/dockerfile:1
#
# Optimized Dockerfile for a Rust Workspace using cargo-chef
# Based on https://github.com/tursodatabase/turso/blob/main/Dockerfile.antithesis
#
# This Dockerfile uses cargo-chef for optimal layer caching in Rust builds.

# This ARG must be declared before the first FROM so it can be used there.
# It defines the target architecture for the build.
ARG BUILD_PLATFORM=linux/amd64

##########################################
## 1️⃣ Chef Stage (cargo-chef)          ##
##########################################

FROM --platform=${BUILD_PLATFORM} lukemathwalker/cargo-chef:0.1.72-rust-1.88.0-slim-bullseye AS chef

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    git \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    libudev-dev \
    zlib1g-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

##########################################
## 2️⃣ Planner Stage                   ##
##########################################

FROM chef AS planner

# Re-declare the ARG for this stage
ARG BUILD_PLATFORM

# Use default target - cargo-chef will handle the right target

# Copy all the source files for dependency planning
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY protocols/ ./protocols/

# Prepare the recipe for building dependencies
RUN cargo chef prepare --recipe-path recipe.json

##########################################
## 3️⃣ Builder Stage                    ##
##########################################

FROM chef AS builder

# Re-declare the ARG for this stage
ARG BUILD_PLATFORM

# Copy the recipe from planner stage
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies using the recipe
RUN PKG_CONFIG_ALLOW_CROSS=1 \
    PROTOC=/usr/bin/protoc \
    RUSTFLAGS="-C target-cpu=generic" \
    cargo chef cook --release --recipe-path recipe.json

# Copy the actual source code
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY protocols/ ./protocols/

# Build the actual application binaries
RUN PKG_CONFIG_ALLOW_CROSS=1 \
    PROTOC=/usr/bin/protoc \
    RUSTFLAGS="-C target-cpu=generic" \
    cargo build --release \
    --package reev-agent \
    --package reev-api \
    --package reev-runner

##########################################
## 4️⃣ Runtime Stage (minimal, secure) ##
##########################################

FROM ubuntu:20.04

# Install runtime dependencies including OpenSSL libraries
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl1.1 \
    libudev1 \
    && rm -rf /var/lib/apt/lists/*

# Create a dedicated non-root user for security
RUN groupadd -r app && \
    useradd -r -u 1000 -g app app

# Set the working directory for the runtime stage
WORKDIR /app

# Copy the compiled binaries from the builder stage
COPY --from=builder /app/target/release/reev-agent /app/reev-agent
COPY --from=builder /app/target/release/reev-api /app/reev-api
COPY --from=builder /app/target/release/reev-runner /app/reev-runner

# Set correct ownership for all application files
RUN chown -R app:app /app

# Expose the service ports for each binary
EXPOSE 8080 9090 9091

# Health check to ensure the API service is responsive
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s \
    CMD curl -f http://localhost:9090/health || exit 1

# Run as the non-root user
USER app

# Default entrypoint can be overridden to run specific binary
ENTRYPOINT ["/app/reev-api"]
