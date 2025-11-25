#!/usr/bin/env bash

# test-docker.sh - Test script for validating Docker builds
# This script tests all Dockerfile variants to ensure they work

set -e

echo "=== Docker Build Test Script ==="
echo "Testing all Dockerfile variants for reev project"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS")
            echo -e "${GREEN}✓ ${message}${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠ ${message}${NC}"
            ;;
        "ERROR")
            echo -e "${RED}✗ ${message}${NC}"
            ;;
        "INFO")
            echo -e "${YELLOW}ℹ ${message}${NC}"
            ;;
    esac
}

# Check if Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        print_status "ERROR" "Docker is not running. Please start Docker daemon."
        echo "On macOS: Start Docker Desktop"
        echo "On Linux: sudo systemctl start docker"
        exit 1
    fi
    print_status "SUCCESS" "Docker daemon is running"
}

# Test Dockerfile build
test_dockerfile() {
    local dockerfile=$1
    local tag=$2
    local description=$3

    echo ""
    print_status "INFO" "Testing ${dockerfile} - ${description}"

    if [[ ! -f "${dockerfile}" ]]; then
        print_status "ERROR" "Dockerfile ${dockerfile} not found"
        return 1
    fi

    echo "Building ${dockerfile} as ${tag}..."
    if docker build -f "${dockerfile}" -t "${tag}" .; then
        print_status "SUCCESS" "${dockerfile} built successfully"

        # Test basic container functionality
        echo "Testing container basic functionality..."
        if docker run --rm "${tag}" --version 2>/dev/null || docker run --rm "${tag}" --help 2>/dev/null; then
            print_status "SUCCESS" "Container runs successfully"
        else
            print_status "WARNING" "Container runs but may have issues"
        fi

        # Clean up
        docker rmi "${tag}" >/dev/null 2>&1 || true
        return 0
    else
        print_status "ERROR" "${dockerfile} build failed"
        return 1
    fi
}

# Main execution
main() {
    echo "Starting Docker build tests..."

    check_docker

    local success_count=0
    local total_count=0

    # Test Dockerfile.github
    ((total_count++))
    if test_dockerfile "Dockerfile.github" "reev-github-test" "Ubuntu-based GitHub simulation"; then
        ((success_count++))
    fi

    # Test Dockerfile.cloudflare
    ((total_count++))
    if test_dockerfile "Dockerfile.cloudflare" "reev-cloudflare-test" "Alpine-based Cloudflare optimized"; then
        ((success_count++))
    fi

    # Test original Dockerfile
    ((total_count++))
    if test_dockerfile "Dockerfile" "reev-original-test" "Original cargo-chef build"; then
        ((success_count++))
    fi

    # Summary
    echo ""
    echo "=== Test Summary ==="
    echo "Successful builds: ${success_count}/${total_count}"

    if [[ ${success_count} -eq ${total_count} ]]; then
        print_status "SUCCESS" "All Dockerfile variants build successfully!"
        echo "Ready for CI/CD deployment!"
    else
        print_status "WARNING" "Some builds failed. Check the output above for details."
        echo "You may need to fix issues before CI/CD deployment."
    fi

    echo ""
    echo "=== Next Steps ==="
    echo "1. If all builds pass: Ready for GitHub Actions"
    echo "2. If some fail: Check CICD.md for troubleshooting"
    echo "3. For deployment: Use build.sh with appropriate DOCKERFILE"
}

# Run main function
main "$@"
