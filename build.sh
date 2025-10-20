#!/usr/bin/env bash

# build.sh - GitHub CI/CD build script for reev project
# Based on Solana's docker build approach with platform support

set -e

# Get script directory
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default build arguments
IMAGE_NAME="${IMAGE_NAME:-reev}"
DOCKERFILE="${DOCKERFILE:-Dockerfile.github}"
BUILD_PLATFORM="${BUILD_PLATFORM:-linux/amd64}"

# Platform detection for M1/M2 Macs
platform_args=()
if [[ $(uname -m) = arm64 ]]; then
    # Ref: https://blog.jaimyn.dev/how-to-build-multi-architecture-docker-images-on-an-m1-mac/#tldr
    platform_args+=(--platform linux/amd64)
fi

echo "Building Docker image: ${IMAGE_NAME}"
echo "Using Dockerfile: ${DOCKERFILE}"
echo "Target platform: ${BUILD_PLATFORM}"

# Build arguments for complex dependencies
docker build "${platform_args[@]}" \
    -f "${here}/${DOCKERFILE}" \
    --build-arg "TARGETPLATFORM=${BUILD_PLATFORM}" \
    --build-arg "BUILD_PLATFORM=${BUILD_PLATFORM}" \
    -t "${IMAGE_NAME}" \
    "${here}"

echo "Build completed successfully: ${IMAGE_NAME}"

# Optional: Push to registry if PUSH_IMAGE is set
if [[ "${PUSH_IMAGE}" == "true" ]]; then
    echo "Pushing image to registry..."
    docker push "${IMAGE_NAME}"
    echo "Push completed successfully"
fi
