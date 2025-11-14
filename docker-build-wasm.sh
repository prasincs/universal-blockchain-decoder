#!/bin/bash
# Docker-based WASM build script
# This script builds WASM artifacts using Docker, ensuring consistent builds
# across all platforms (Linux, macOS, Windows).

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🐳 Universal Blockchain Decoder - Docker WASM Build${NC}"
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Error: Docker is not installed${NC}"
    echo "Please install Docker from: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo -e "${YELLOW}⚠️  Warning: docker-compose not found, using 'docker compose' instead${NC}"
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

# Parse command line arguments
BUILD_TYPE="${1:-release}"
SERVE="${2:-false}"

case "$BUILD_TYPE" in
    release)
        echo -e "${GREEN}📦 Building WASM in release mode...${NC}"
        $COMPOSE_CMD run --rm wasm-builder
        ;;
    dev)
        echo -e "${YELLOW}🔧 Starting development environment...${NC}"
        $COMPOSE_CMD run --rm wasm-dev
        ;;
    shell)
        echo -e "${BLUE}🐚 Opening interactive shell in development container...${NC}"
        $COMPOSE_CMD run --rm wasm-dev /bin/bash
        ;;
    full)
        echo -e "${GREEN}📦 Building full WASM and copying to /wasm directory...${NC}"
        $COMPOSE_CMD run --rm wasm-builder bash /app/rebuild-wasm.sh
        ;;
    clean)
        echo -e "${YELLOW}🧹 Cleaning up Docker images and volumes...${NC}"
        $COMPOSE_CMD down -v
        docker rmi universal-decoder-wasm-builder:latest 2>/dev/null || true
        docker rmi universal-decoder-wasm-dev:latest 2>/dev/null || true
        echo -e "${GREEN}✅ Cleanup complete${NC}"
        exit 0
        ;;
    serve)
        echo -e "${BLUE}🌐 Starting web server for WASM demo...${NC}"
        echo -e "${GREEN}📍 WASM demo will be available at:${NC}"
        echo -e "   ${BLUE}http://localhost:8080${NC} (from crates/universal-decoder-wasm/www)"
        echo -e "   ${BLUE}http://localhost:8081${NC} (from /wasm directory)"
        echo ""
        echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
        $COMPOSE_CMD up wasm-server wasm-server-root
        exit 0
        ;;
    build-and-serve)
        echo -e "${GREEN}📦 Building WASM and starting web server...${NC}"
        $COMPOSE_CMD run --rm wasm-builder
        echo ""
        echo -e "${BLUE}🌐 Starting web server...${NC}"
        echo -e "${GREEN}📍 WASM demo available at: ${BLUE}http://localhost:8080${NC}"
        echo ""
        echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
        $COMPOSE_CMD up wasm-server
        exit 0
        ;;
    *)
        echo -e "${RED}❌ Unknown build type: $BUILD_TYPE${NC}"
        echo ""
        echo "Usage: $0 [BUILD_TYPE]"
        echo ""
        echo "Build types:"
        echo "  release          Build WASM in release mode (default)"
        echo "  dev              Start development environment"
        echo "  shell            Open interactive shell in container"
        echo "  full             Build WASM and copy to /wasm directory"
        echo "  serve            Start web server to test WASM demo"
        echo "  build-and-serve  Build WASM and start web server"
        echo "  clean            Remove Docker images and volumes"
        echo ""
        echo "Examples:"
        echo "  $0                    # Build WASM (release mode)"
        echo "  $0 shell              # Open shell for manual commands"
        echo "  $0 build-and-serve    # Build and test locally"
        echo "  $0 clean              # Clean up Docker artifacts"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}✅ Build complete!${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Test locally:  $0 serve"
echo "  2. Check output:  ls -lh crates/universal-decoder-wasm/www/pkg/"
echo "  3. Deploy:        See wasm/DEPLOY.md"
echo ""
