#!/bin/bash

# 🌊 Drift Registry Test Script
# This script tests the Drift registry with Docker and Podman

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REGISTRY_URL="localhost:5000"
TEST_USERNAME="admin"
TEST_PASSWORD="changeme"
TEST_IMAGE="alpine:latest"
TEST_REPO="drift-test"
TEST_TAG="v1.0.0"

echo -e "${BLUE}🌊 Drift Registry Test Suite${NC}"
echo "=================================="

# Function to print status
print_status() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if registry is running
check_registry() {
    print_status "Checking if Drift registry is running..."

    if curl -f -s "http://${REGISTRY_URL}/v2/" > /dev/null; then
        print_success "Registry is accessible at http://${REGISTRY_URL}"
    else
        print_error "Registry is not accessible. Please start it with: docker compose up -d"
        exit 1
    fi

    # Check API version
    local api_response=$(curl -s "http://${REGISTRY_URL}/v2/")
    if echo "$api_response" | grep -q "drift"; then
        print_success "Drift registry API is responding correctly"
    else
        print_warning "Unexpected API response: $api_response"
    fi
}

# Test Docker operations
test_docker() {
    print_status "Testing Docker operations..."

    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
        return 1
    fi

    # Pull test image
    print_status "Pulling test image: $TEST_IMAGE"
    docker pull $TEST_IMAGE

    # Tag for our registry
    local full_tag="${REGISTRY_URL}/${TEST_REPO}:${TEST_TAG}"
    print_status "Tagging image: $full_tag"
    docker tag $TEST_IMAGE $full_tag

    # Login to registry
    print_status "Logging into registry..."
    echo "$TEST_PASSWORD" | docker login $REGISTRY_URL -u $TEST_USERNAME --password-stdin

    # Push image
    print_status "Pushing image to registry..."
    docker push $full_tag

    # Remove local image
    print_status "Removing local images..."
    docker rmi $full_tag $TEST_IMAGE || true

    # Pull from registry
    print_status "Pulling image from registry..."
    docker pull $full_tag

    # Verify image
    if docker image inspect $full_tag > /dev/null 2>&1; then
        print_success "Docker operations completed successfully"
        return 0
    else
        print_error "Failed to verify pulled image"
        return 1
    fi
}

# Test Podman operations
test_podman() {
    print_status "Testing Podman operations..."

    # Check if Podman is available
    if ! command -v podman &> /dev/null; then
        print_warning "Podman is not installed, skipping Podman tests"
        return 0
    fi

    # Pull test image
    print_status "Pulling test image with Podman: $TEST_IMAGE"
    podman pull $TEST_IMAGE

    # Tag for our registry
    local full_tag="${REGISTRY_URL}/${TEST_REPO}:podman-${TEST_TAG}"
    print_status "Tagging image: $full_tag"
    podman tag $TEST_IMAGE $full_tag

    # Login to registry
    print_status "Logging into registry with Podman..."
    echo "$TEST_PASSWORD" | podman login $REGISTRY_URL -u $TEST_USERNAME --password-stdin

    # Push image
    print_status "Pushing image to registry with Podman..."
    podman push $full_tag

    # Remove local image
    print_status "Removing local images..."
    podman rmi $full_tag $TEST_IMAGE || true

    # Pull from registry
    print_status "Pulling image from registry with Podman..."
    podman pull $full_tag

    # Verify image
    if podman image inspect $full_tag > /dev/null 2>&1; then
        print_success "Podman operations completed successfully"
        return 0
    else
        print_error "Failed to verify pulled image with Podman"
        return 1
    fi
}

# Test registry API endpoints
test_api() {
    print_status "Testing registry API endpoints..."

    # Test catalog endpoint
    print_status "Testing repository catalog..."
    local catalog_response=$(curl -s -u "${TEST_USERNAME}:${TEST_PASSWORD}" "http://${REGISTRY_URL}/v2/_catalog")
    if echo "$catalog_response" | grep -q "repositories"; then
        print_success "Catalog endpoint working"
        echo "  Repositories: $(echo "$catalog_response" | jq -r '.repositories[]' 2>/dev/null || echo 'Unable to parse')"
    else
        print_error "Catalog endpoint failed"
    fi

    # Test tags list
    print_status "Testing tags list for $TEST_REPO..."
    local tags_response=$(curl -s -u "${TEST_USERNAME}:${TEST_PASSWORD}" "http://${REGISTRY_URL}/v2/${TEST_REPO}/tags/list")
    if echo "$tags_response" | grep -q "tags"; then
        print_success "Tags endpoint working"
        echo "  Tags: $(echo "$tags_response" | jq -r '.tags[]' 2>/dev/null || echo 'Unable to parse')"
    else
        print_warning "Tags endpoint failed (may be expected if no images pushed yet)"
    fi
}

# Test Bolt API endpoints
test_bolt_api() {
    print_status "Testing Bolt API endpoints..."

    # Test profiles list
    print_status "Testing Bolt profiles endpoint..."
    local profiles_response=$(curl -s "http://${REGISTRY_URL}/v1/profiles")
    if echo "$profiles_response" | grep -q "results"; then
        print_success "Bolt profiles endpoint working"
        local profile_count=$(echo "$profiles_response" | jq -r '.total' 2>/dev/null || echo 'Unknown')
        echo "  Total profiles: $profile_count"
    else
        print_error "Bolt profiles endpoint failed"
    fi

    # Test plugins list
    print_status "Testing Bolt plugins endpoint..."
    local plugins_response=$(curl -s "http://${REGISTRY_URL}/v1/plugins")
    if echo "$plugins_response" | grep -q "results"; then
        print_success "Bolt plugins endpoint working"
        local plugin_count=$(echo "$plugins_response" | jq -r '.total' 2>/dev/null || echo 'Unknown')
        echo "  Total plugins: $plugin_count"
    else
        print_error "Bolt plugins endpoint failed"
    fi

    # Test metrics
    print_status "Testing Bolt metrics endpoint..."
    local metrics_response=$(curl -s "http://${REGISTRY_URL}/v1/metrics")
    if echo "$metrics_response" | grep -q "total_profiles"; then
        print_success "Bolt metrics endpoint working"
    else
        print_error "Bolt metrics endpoint failed"
    fi
}

# Test web UI
test_web_ui() {
    print_status "Testing Web UI..."

    if curl -f -s "http://localhost:5001/" > /dev/null; then
        print_success "Web UI is accessible at http://localhost:5001"
    else
        print_error "Web UI is not accessible"
    fi
}

# Test health endpoints
test_health() {
    print_status "Testing health endpoints..."

    # Health check
    if curl -f -s "http://${REGISTRY_URL}/health" | grep -q "OK"; then
        print_success "Health endpoint working"
    else
        print_error "Health endpoint failed"
    fi

    # Readiness check
    if curl -f -s "http://${REGISTRY_URL}/readyz" | grep -q "Ready"; then
        print_success "Readiness endpoint working"
    else
        print_error "Readiness endpoint failed"
    fi

    # Metrics
    if curl -f -s "http://${REGISTRY_URL}/metrics" > /dev/null; then
        print_success "Metrics endpoint working"
    else
        print_error "Metrics endpoint failed"
    fi
}

# Cleanup function
cleanup() {
    print_status "Cleaning up test images..."

    # Docker cleanup
    if command -v docker &> /dev/null; then
        docker rmi "${REGISTRY_URL}/${TEST_REPO}:${TEST_TAG}" 2>/dev/null || true
        docker rmi $TEST_IMAGE 2>/dev/null || true
    fi

    # Podman cleanup
    if command -v podman &> /dev/null; then
        podman rmi "${REGISTRY_URL}/${TEST_REPO}:podman-${TEST_TAG}" 2>/dev/null || true
        podman rmi $TEST_IMAGE 2>/dev/null || true
    fi

    print_success "Cleanup completed"
}

# Performance test
test_performance() {
    print_status "Running basic performance test..."

    local start_time=$(date +%s.%N)
    curl -s "http://${REGISTRY_URL}/v2/" > /dev/null
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc 2>/dev/null || echo "unknown")

    print_success "API response time: ${duration}s"
}

# Main test execution
main() {
    echo -e "${BLUE}Starting Drift Registry tests...${NC}"
    echo

    # Run tests
    check_registry
    test_health
    test_api
    test_bolt_api
    test_web_ui
    test_performance

    echo
    print_status "Starting container engine tests..."

    # Container engine tests
    if test_docker; then
        DOCKER_SUCCESS=true
    else
        DOCKER_SUCCESS=false
    fi

    if test_podman; then
        PODMAN_SUCCESS=true
    else
        PODMAN_SUCCESS=false
    fi

    # Summary
    echo
    echo -e "${BLUE}Test Summary${NC}"
    echo "============"

    if [ "$DOCKER_SUCCESS" = true ]; then
        print_success "Docker integration: PASSED"
    else
        print_error "Docker integration: FAILED"
    fi

    if [ "$PODMAN_SUCCESS" = true ]; then
        print_success "Podman integration: PASSED"
    else
        print_warning "Podman integration: SKIPPED or FAILED"
    fi

    # Optional cleanup
    if [ "${CLEANUP:-yes}" = "yes" ]; then
        echo
        cleanup
    fi

    echo
    print_success "🌊 Drift Registry testing completed!"
    echo "Web UI: http://localhost:5001"
    echo "Registry API: http://localhost:5000/v2/"
    echo "MinIO Console: http://localhost:9001"
    echo "Grafana: http://localhost:3000"
}

# Handle script arguments
case "${1:-}" in
    --no-cleanup)
        CLEANUP=no
        main
        ;;
    --docker-only)
        check_registry
        test_docker
        ;;
    --podman-only)
        check_registry
        test_podman
        ;;
    --api-only)
        check_registry
        test_api
        test_bolt_api
        ;;
    --help|-h)
        echo "Usage: $0 [options]"
        echo "Options:"
        echo "  --no-cleanup    Don't clean up test images"
        echo "  --docker-only   Test Docker integration only"
        echo "  --podman-only   Test Podman integration only"
        echo "  --api-only      Test API endpoints only"
        echo "  --help          Show this help"
        exit 0
        ;;
    *)
        main
        ;;
esac