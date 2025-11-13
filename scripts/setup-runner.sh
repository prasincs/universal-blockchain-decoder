#!/bin/bash
# GitHub Actions Self-Hosted Runner - Automated Setup Script
# For Ubuntu 22.04 LTS (adjust for other distros)
#
# Usage:
#   sudo ./setup-runner.sh <registration-token>
#
# Get token from:
#   https://github.com/prasincs/universal-blockchain-decoder/settings/actions/runners/new

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
echo_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
echo_error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo_error "Please run as root (sudo ./setup-runner.sh)"
fi

# Check for registration token
if [ -z "$1" ]; then
    echo_error "Usage: sudo ./setup-runner.sh <registration-token>"
fi

RUNNER_TOKEN="$1"
RUNNER_VERSION="2.311.0"
RUNNER_NAME="${2:-bare-metal-linux-01}"
REPO_URL="https://github.com/prasincs/universal-blockchain-decoder"

echo_info "Starting GitHub Actions Runner setup..."
echo_info "Runner version: $RUNNER_VERSION"
echo_info "Runner name: $RUNNER_NAME"

# Step 1: Update system
echo_info "Updating system packages..."
apt update && apt upgrade -y

# Step 2: Install dependencies
echo_info "Installing dependencies..."
apt install -y \
    curl wget git build-essential libssl-dev pkg-config \
    jq ca-certificates gnupg lsb-release \
    docker.io docker-compose

# Step 3: Create github-runner user
echo_info "Creating github-runner user..."
if id "github-runner" &>/dev/null; then
    echo_warn "User github-runner already exists, skipping..."
else
    useradd -m -s /bin/bash github-runner
    usermod -aG docker github-runner
fi

# Step 4: Install Rust as github-runner
echo_info "Installing Rust toolchain..."
sudo -u github-runner bash -c '
    if [ ! -d "$HOME/.cargo" ]; then
        curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        rustup toolchain install stable nightly
        rustup component add rustfmt clippy
    else
        echo "Rust already installed"
    fi
'

# Step 5: Download and configure runner
echo_info "Downloading GitHub Actions runner..."
sudo -u github-runner bash -c "
    cd ~
    mkdir -p actions-runner && cd actions-runner

    if [ ! -f 'config.sh' ]; then
        wget -q https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz
        tar xzf ./actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz
        rm actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz
    fi
"

# Step 6: Configure runner
echo_info "Configuring runner..."
sudo -u github-runner bash << EOF
    cd ~/actions-runner
    ./config.sh \
        --url ${REPO_URL} \
        --token ${RUNNER_TOKEN} \
        --name ${RUNNER_NAME} \
        --labels "self-hosted,linux,bare-metal,rust" \
        --work _work \
        --unattended
EOF

# Step 7: Install as service
echo_info "Installing runner as system service..."
cd /home/github-runner/actions-runner
./svc.sh install github-runner

# Step 8: Start service
echo_info "Starting runner service..."
./svc.sh start

# Enable on boot
systemctl enable actions.runner.*.service

# Step 9: Configure firewall
echo_info "Configuring firewall..."
if command -v ufw &> /dev/null; then
    ufw --force enable
    ufw allow 22/tcp
    ufw allow out 443/tcp
    ufw default deny incoming
    ufw default allow outgoing
fi

# Step 10: Set up monitoring
echo_info "Creating monitoring script..."
cat > /home/github-runner/monitor.sh << 'MONITOR_SCRIPT'
#!/bin/bash
# Runner health check

if ! systemctl is-active --quiet actions.runner.*.service; then
    echo "❌ Runner service is down!"
    systemctl restart actions.runner.*.service
fi

FREE_SPACE=$(df /home | tail -1 | awk '{print $4}')
if [ "$FREE_SPACE" -lt 10485760 ]; then
    echo "⚠️ Low disk space: $(df -h /home | tail -1)"
fi

echo "✅ Runner health check passed at $(date)"
MONITOR_SCRIPT

chmod +x /home/github-runner/monitor.sh
chown github-runner:github-runner /home/github-runner/monitor.sh

# Add to crontab
sudo -u github-runner bash -c '
    (crontab -l 2>/dev/null; echo "*/15 * * * * /home/github-runner/monitor.sh >> /home/github-runner/monitor.log 2>&1") | crontab -
'

# Step 11: Verify installation
echo_info "Verifying installation..."
sleep 5

if systemctl is-active --quiet actions.runner.*.service; then
    echo_info "✅ Runner service is running!"
else
    echo_error "❌ Runner service failed to start. Check logs: journalctl -u actions.runner.*.service"
fi

# Print summary
echo ""
echo_info "=========================================="
echo_info "GitHub Actions Runner Setup Complete!"
echo_info "=========================================="
echo_info "Runner name: $RUNNER_NAME"
echo_info "Service status: $(systemctl is-active actions.runner.*.service)"
echo_info ""
echo_info "Next steps:"
echo_info "1. Verify runner online: $REPO_URL/settings/actions/runners"
echo_info "2. Update workflow files to use: runs-on: [self-hosted, linux, bare-metal]"
echo_info "3. Monitor logs: sudo journalctl -u actions.runner.*.service -f"
echo_info ""
echo_info "Service commands:"
echo_info "  Start:   sudo /home/github-runner/actions-runner/svc.sh start"
echo_info "  Stop:    sudo /home/github-runner/actions-runner/svc.sh stop"
echo_info "  Status:  sudo /home/github-runner/actions-runner/svc.sh status"
echo_info "  Restart: sudo /home/github-runner/actions-runner/svc.sh restart"
echo_info ""
echo_info "Documentation: docs/SELF_HOSTED_RUNNER_LINUX.md"
echo_info "=========================================="
