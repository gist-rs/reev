# CI/CD Setup and Troubleshooting

This document covers the CI/CD setup for reev project, including Docker builds for different environments with **current working status**.

## Dockerfiles - Current Status

### 1. Dockerfile (original) ✅ WORKING
- **Purpose**: General-purpose build with cargo-chef
- **Base**: Ubuntu 20.04 with cargo-chef
- **Status**: ✅ FULLY FUNCTIONAL - Tested and validated
- **Features**: 
  - Optimized layer caching with cargo-chef
  - Non-root user for security
  - Health checks included
  - All packages build: reev-agent, reev-api, reev-runner
- **Issues**: None

### 2. Dockerfile.github ❌ NOT WORKING
- **Purpose**: Ubuntu-based build simulating GitHub Actions runner
- **Base**: Ubuntu 20.04
- **Status**: ❌ COMPILATION FAILURES
- **Features**: 
  - Comprehensive dependency installation for Solana, Turso, OpenSSL
  - cargo-chef for optimized layer caching
  - Non-root user for security
- **Blocking Issues**:
  - AEGIS crypto library compilation fails with `-mtune=native` flag
  - Multiple RUSTFLAGS override attempts unsuccessful
  - Container cross-compilation incompatibility

### 3. Dockerfile.cloudflare ❌ NOT WORKING
- **Purpose**: Alpine-based build for Cloudflare Containers
- **Base**: Alpine Linux + Rust
- **Status**: ❌ BUILD FAILURES
- **Features**:
  - Uses cargo-zigbuild for static linking
  - Minimal scratch runtime image
  - Cross-compilation support (amd64/arm64)
- **Blocking Issues**:
  - Alpine package incompatibilities (`libudev-dev` vs `eudev-dev`)
  - Static linking conflicts with crypto libraries
  - cargo-zigbuild workspace dependency issues

## Build Scripts

### build.sh ✅ WORKING
- **Status**: ✅ FUNCTIONAL - Works with default Dockerfile
- **Features**:
  - GitHub CI/CD compatible
  - Platform detection for M1/M2 Macs
  - Configurable image name and Dockerfile
  - Optional push to registry
- **Limitation**: Only works with default Dockerfile, not github/cloudflare variants

### test-docker.sh ⚠️ PARTIAL
- **Status**: ⚠️ WORKS FOR TESTING - 1/3 Dockerfiles pass
- **Features**:
  - Tests all Dockerfile variants
  - Color-coded output for debugging
  - Automatic cleanup of test images
- **Current Results**: Only original Dockerfile passes testing

## Current Working Configuration

### What Works Right Now
```bash
# Build working image
docker build -f Dockerfile -t reev:latest .

# Test container functionality
docker run -d -p 3001:3001 -v $(pwd)/benchmarks:/app/benchmarks -v $(pwd)/db:/app/db reev:latest

# Verify health
curl http://localhost:3001/api/v1/health
# Response: {"status":"healthy","timestamp":"...","version":"0.1.0"}
```

### Build Results Summary
```
=== Docker Build Test Results ===
✅ Dockerfile (original): PASS - Full functionality verified
❌ Dockerfile.github: FAIL - AEGIS compilation errors
❌ Dockerfile.cloudflare: FAIL - Alpine static linking issues
```

## Common Issues and Solutions

### Fixed Issues ✅

#### Missing Agent Module
**Problem**: `reev_lib::agent` module not found
**Solution**: Re-enabled agent module in `reev-lib/src/lib.rs`
**Status**: ✅ RESOLVED

#### OpenSSL Dependencies
**Problem**: Missing OpenSSL libraries during build
**Solution**: Install `libssl-dev` (Ubuntu)
**Status**: ✅ RESOLVED

#### Solana Dependencies
**Problem**: Missing system libraries for Solana SDK
**Solution**: Install:
- `libudev-dev` (Ubuntu)
- `zlib1g-dev` (Ubuntu)
- `protobuf-compiler` and `libprotobuf-dev`
**Status**: ✅ RESOLVED

### Unresolved Issues ❌

#### AEGIS Crypto Library Compilation
**Problem**: `-mtune=native` flag incompatible with container builds
**Affects**: Dockerfile.github
**Attempts**: 
- `RUSTFLAGS="-C target-cpu=generic"`
- `CARGO_PROFILE_RELEASE_LTO=off`
- CC/CXX override attempts
**Status**: ❌ NOT RESOLVED

#### Alpine Package Incompatibility
**Problem**: `libudev-dev` not available in Alpine
**Affects**: Dockerfile.cloudflare
**Attempts**: Changed to `eudev-dev`, still issues with static linking
**Status**: ❌ NOT RESOLVED

#### Static Linking Conflicts
**Problem**: Multiple crypto library conflicts in static build
**Affects**: Dockerfile.cloudflare
**Status**: ❌ NOT RESOLVED

## Testing Locally

### Working Test
```bash
# Test only working Dockerfile
docker build -f Dockerfile -t reev-working .

# Full functionality test
docker run --rm reev-working /app/reev-api --version
```

### Failed Tests
```bash
# These will fail currently:
docker build -f Dockerfile.github -t reev-github .     # FAILS
docker build -f Dockerfile.cloudflare -t reev-cloudflare . # FAILS
./test-docker.sh                                       # 1/3 PASS ONLY
```

## GitHub Actions Workflow - UPDATED

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
          # Use working Dockerfile only
          export IMAGE_NAME=ghcr.io/${{ github.repository }}:latest
          export DOCKERFILE="Dockerfile"  # NOT Dockerfile.github
          export PUSH_IMAGE=true
          ./build.sh
```

## Deployment Targets - CURRENT STATUS

### 1. GitHub Container Registry ✅ AVAILABLE
- **Dockerfile**: `Dockerfile` (original working version)
- **Runtime**: Ubuntu environment
- **Status**: ✅ READY FOR PRODUCTION
- **Features**: Full functionality validated

### 2. Cloudflare Containers ❌ NOT AVAILABLE
- **Dockerfile**: `Dockerfile.cloudflare` (failing)
- **Runtime**: Minimal scratch image
- **Status**: ❌ BLOCKED BY BUILD ISSUES
- **Features**: Static linking not working

## Troubleshooting Checklist

### Pre-Build ✅
- [x] OpenSSL libraries installed
- [x] Protobuf compiler available
- [x] Rust targets added
- [x] Agent module enabled

### Post-Build ✅ (for working Dockerfile)
- [x] Non-root user permissions
- [x] Health check endpoints verified
- [x] Database connectivity works
- [x] Benchmarks sync functional

### Failed Checks ❌
- [ ] AEGIS compilation resolved
- [ ] Alpine packages working
- [ ] Static linking functional
- [ ] Multi-architecture builds
- [ ] Cloudflare compatibility

## Production Deployment Instructions

### For Immediate Use
```bash
# Build and deploy working configuration
export IMAGE_NAME=your-registry/reev:latest
export DOCKERFILE="Dockerfile"  # Use working version
export PUSH_IMAGE=true
./build.sh

# Deploy to your preferred platform
docker push $IMAGE_NAME
```

### Monitoring
```bash
# Health check after deployment
curl http://your-domain/api/v1/health
# Expected: {"status":"healthy","timestamp":"...","version":"0.1.0"}
```

## Future Improvements

### Immediate Priorities
1. **Fix AEGIS compilation** - Research alternative crypto libraries or build flags
2. **Resolve Alpine issues** - Find compatible packages or different base image
3. **Multi-architecture support** - Add amd64/arm64 builds for working Dockerfile

### Long-term Goals
1. Automated testing in different environments
2. Optimized layer caching strategies
3. Security scanning integration
4. Smaller final image sizes
5. Container registry CI/CD pipeline

## Workarounds Until Issues Resolved

### For GitHub Actions
- Use `Dockerfile` instead of `Dockerfile.github`
- Works perfectly, just not Ubuntu-simulation specific

### For Cloudflare Deployment
- Use working Ubuntu-based image until Alpine build fixed
- Larger image size but fully functional

### For Local Development
- Default Dockerfile provides full functionality
- No need for alternative variants until issues resolved