# CI/CD Setup and Troubleshooting

This document covers the CI/CD setup for the reev project, including Docker builds for different environments.

## Dockerfiles

### 1. Dockerfile.github
- **Purpose**: Ubuntu-based build simulating GitHub Actions runner
- **Base**: Ubuntu 20.04
- **Features**: 
  - Comprehensive dependency installation for Solana, Turso, OpenSSL
  - Uses cargo-chef for optimized layer caching
  - Non-root user for security
  - Health checks included

### 2. Dockerfile.cloudflare
- **Purpose**: Alpine-based build for Cloudflare Containers
- **Base**: Alpine Linux + Rust
- **Features**:
  - Uses cargo-zigbuild for static linking
  - Minimal scratch runtime image
  - Cross-compilation support (amd64/arm64)
  - Optimized for Cloudflare's container runtime

### 3. Dockerfile (original)
- **Purpose**: General-purpose build with cargo-chef
- **Base**: Ubuntu with cargo-chef
- **Status**: Works on Mac, needs Ubuntu testing

## Build Scripts

### build.sh
- GitHub CI/CD compatible build script
- Platform detection for M1/M2 Macs
- Configurable image name and Dockerfile
- Optional push to registry

### test-docker.sh
- Comprehensive test script for all Dockerfile variants
- Validates Docker builds and basic container functionality
- Color-coded output for easy debugging
- Automatic cleanup of test images

## Common Issues and Solutions

### OpenSSL Dependencies
**Problem**: Missing OpenSSL libraries during build
**Solution**: Install `libssl-dev` (Ubuntu) or `openssl-dev` (Alpine)

### Solana Dependencies
**Problem**: Missing system libraries for Solana SDK
**Solution**: Install:
- `libudev-dev` (Ubuntu) / `libudev-dev` (Alpine)
- `zlib1g-dev` (Ubuntu) / `zlib-dev` (Alpine)
- `protobuf-compiler` and `libprotobuf-dev`

### Turso Dependencies
**Problem**: SQLite and libsql compilation issues
**Solution**: Ensure pkg-config and protobuf tools are available

### Cloudflare Container Issues
**Problem**: Dynamic linking failures in Cloudflare runtime
**Solution**: Use Alpine + cargo-zigbuild for static linking

## Testing Locally

### Quick Test All Variants
```bash
# Test all Dockerfile variants with comprehensive validation
./test-docker.sh
```

### Test Individual Builds
```bash
# Test Ubuntu-based build (simulates GitHub runner)
docker build -f Dockerfile.github -t reev-github .

# Test Alpine-based build (Cloudflare optimized)
docker build -f Dockerfile.cloudflare -t reev-cloudflare .

# Test original cargo-chef build
docker build -f Dockerfile -t reev-original .
```

### Test in Ubuntu Container
```bash
# Run Ubuntu container with Docker inside
docker run -it --privileged -v $(pwd):/workspace ubuntu:20.04 bash
# Inside container:
cd /workspace
apt-get update && apt-get install -y docker.io
./build.sh
```

## GitHub Actions Workflow

```yaml
name: Build and Deploy
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Docker image
        run: |
          export IMAGE_NAME=ghcr.io/${{ github.repository }}:latest
          export PUSH_IMAGE=true
          ./build.sh
```

## Deployment Targets

### 1. GitHub Container Registry
- Built using Dockerfile.github
- Ubuntu runtime environment
- Full feature support

### 2. Cloudflare Containers
- Built using Dockerfile.cloudflare
- Minimal runtime footprint
- Static linking for compatibility

## Troubleshooting Checklist

- [ ] Check OpenSSL libraries are installed
- [ ] Verify protobuf compiler is available
- [ ] Ensure Rust target is added
- [ ] Test on both Ubuntu and Alpine if possible
- [ ] Validate static linking for Cloudflare
- [ ] Check non-root user permissions
- [ ] Verify health check endpoints

## Future Improvements

1. Multi-architecture builds (amd64/arm64)
2. Automated testing in different environments
3. Optimized layer caching strategies
4. Security scanning integration
5. Smaller final image sizes