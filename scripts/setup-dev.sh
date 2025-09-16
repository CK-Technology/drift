#!/bin/bash

# 🌊 Drift Registry - Development Setup Script

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🌊 Drift Registry - Development Setup${NC}"
echo "====================================="

# Check prerequisites
check_prerequisites() {
    echo -e "${BLUE}[SETUP]${NC} Checking prerequisites..."

    # Rust
    if ! command -v rustc &> /dev/null; then
        echo -e "${RED}[ERROR]${NC} Rust is not installed. Install from https://rustup.rs/"
        exit 1
    fi

    local rust_version=$(rustc --version | cut -d' ' -f2)
    echo -e "${GREEN}[OK]${NC} Rust $rust_version"

    # Cargo
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}[ERROR]${NC} Cargo is not installed"
        exit 1
    fi

    # Docker
    if ! command -v docker &> /dev/null; then
        echo -e "${YELLOW}[WARN]${NC} Docker is not installed (optional for testing)"
    else
        echo -e "${GREEN}[OK]${NC} Docker $(docker --version | cut -d' ' -f3 | tr -d ',')"
    fi

    # Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        echo -e "${YELLOW}[WARN]${NC} Docker Compose is not installed (optional for testing)"
    else
        echo -e "${GREEN}[OK]${NC} Docker Compose available"
    fi
}

# Setup development environment
setup_dev_env() {
    echo -e "${BLUE}[SETUP]${NC} Setting up development environment..."

    # Install additional Rust tools
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${BLUE}[INSTALL]${NC} Installing cargo-watch for hot reload..."
        cargo install cargo-watch
    fi

    if ! command -v cargo-leptos &> /dev/null; then
        echo -e "${BLUE}[INSTALL]${NC} Installing cargo-leptos for frontend development..."
        cargo install cargo-leptos
    fi

    # Create data directory
    mkdir -p data
    echo -e "${GREEN}[OK]${NC} Created data directory"

    # Generate default config if it doesn't exist
    if [ ! -f drift.toml ]; then
        echo -e "${BLUE}[CONFIG]${NC} Creating default configuration..."
        cat > drift.toml << EOF
[server]
bind_addr = "127.0.0.1:5000"
ui_addr = "127.0.0.1:5001"

[auth]
mode = "basic"
jwt_secret = "dev-secret-change-in-production"

[auth.basic]
users = ["admin:admin", "dev:dev"]

[storage]
backend = "fs"
path = "./data"

[registry]
max_upload_size_mb = 100
rate_limit_per_hour = 1000

[bolt]
enable_profile_validation = true
enable_plugin_sandbox = true
auto_update_profiles = false
EOF
        echo -e "${GREEN}[OK]${NC} Created drift.toml"
    fi
}

# Build the project
build_project() {
    echo -e "${BLUE}[BUILD]${NC} Building Drift registry..."

    # Build in debug mode for development
    cargo build

    echo -e "${GREEN}[OK]${NC} Build completed"
}

# Start development services
start_dev_services() {
    echo -e "${BLUE}[SERVICES]${NC} Starting development services..."

    # Start MinIO for S3 testing
    if command -v docker &> /dev/null; then
        echo -e "${BLUE}[DOCKER]${NC} Starting MinIO container..."
        docker run -d \
            --name drift-minio-dev \
            -p 9000:9000 \
            -p 9001:9001 \
            -e "MINIO_ROOT_USER=driftdev" \
            -e "MINIO_ROOT_PASSWORD=driftdev123" \
            -v minio-dev-data:/data \
            minio/minio server /data --console-address ":9001" || echo "MinIO already running"

        echo -e "${GREEN}[OK]${NC} MinIO started at http://localhost:9001 (driftdev/driftdev123)"
    fi
}

# Create VSCode configuration
setup_vscode() {
    if [ -d ".vscode" ] || [ ! -t 0 ]; then
        return
    fi

    read -p "Setup VSCode configuration? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        mkdir -p .vscode

        # Launch configuration
        cat > .vscode/launch.json << EOF
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Drift",
            "cargo": {
                "args": ["build", "--bin=drift"],
                "filter": {
                    "name": "drift",
                    "kind": "bin"
                }
            },
            "args": ["--config", "drift.toml"],
            "cwd": "\${workspaceFolder}",
            "env": {
                "RUST_LOG": "drift=debug,tower_http=debug"
            }
        }
    ]
}
EOF

        # Settings
        cat > .vscode/settings.json << EOF
{
    "rust-analyzer.linkedProjects": ["./Cargo.toml"],
    "rust-analyzer.cargo.features": "all",
    "files.watcherExclude": {
        "**/target/**": true
    }
}
EOF

        # Extensions
        cat > .vscode/extensions.json << EOF
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "vadimcn.vscode-lldb",
        "serayuzgur.crates",
        "tamasfe.even-better-toml"
    ]
}
EOF

        echo -e "${GREEN}[OK]${NC} VSCode configuration created"
    fi
}

# Create development scripts
create_dev_scripts() {
    mkdir -p scripts

    # Hot reload script
    cat > scripts/dev.sh << 'EOF'
#!/bin/bash
echo "🌊 Starting Drift in development mode with hot reload..."
RUST_LOG=drift=debug,tower_http=debug cargo watch -x "run -- --config drift.toml"
EOF

    # Test script
    cat > scripts/test.sh << 'EOF'
#!/bin/bash
echo "🧪 Running tests..."
cargo test --workspace
EOF

    # Lint script
    cat > scripts/lint.sh << 'EOF'
#!/bin/bash
echo "🔍 Running lints..."
cargo fmt --all
cargo clippy --all-targets -- -D warnings
EOF

    # Frontend development script
    cat > scripts/frontend.sh << 'EOF'
#!/bin/bash
echo "🎨 Starting frontend development server..."
cargo leptos watch
EOF

    chmod +x scripts/*.sh
    echo -e "${GREEN}[OK]${NC} Development scripts created in ./scripts/"
}

# Print next steps
print_next_steps() {
    echo
    echo -e "${GREEN}🎉 Development environment setup complete!${NC}"
    echo
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. Start development server:"
    echo "   ./scripts/dev.sh"
    echo
    echo "2. Or run manually:"
    echo "   cargo run -- --config drift.toml"
    echo
    echo "3. Access the registry:"
    echo "   - API: http://localhost:5000/v2/"
    echo "   - Web UI: http://localhost:5001/"
    echo "   - MinIO: http://localhost:9001/ (driftdev/driftdev123)"
    echo
    echo "4. Run tests:"
    echo "   ./test-registry.sh"
    echo
    echo "5. Development tools:"
    echo "   ./scripts/test.sh    - Run unit tests"
    echo "   ./scripts/lint.sh    - Run linter and formatter"
    echo "   ./scripts/frontend.sh - Frontend hot reload"
    echo
    echo -e "${YELLOW}Note:${NC} Edit drift.toml to configure storage, auth, and other settings"
}

# Main execution
main() {
    check_prerequisites
    setup_dev_env
    build_project
    start_dev_services
    setup_vscode
    create_dev_scripts
    print_next_steps
}

# Handle arguments
case "${1:-}" in
    --minimal)
        check_prerequisites
        setup_dev_env
        print_next_steps
        ;;
    --build-only)
        build_project
        ;;
    --help|-h)
        echo "Usage: $0 [options]"
        echo "Options:"
        echo "  --minimal     Setup only, no building or services"
        echo "  --build-only  Only build the project"
        echo "  --help        Show this help"
        exit 0
        ;;
    *)
        main
        ;;
esac