# Linux Self-Hosted GitHub Actions Runner - Bare Metal Guide

## Overview

This guide covers setting up a **secure, production-ready** GitHub Actions self-hosted runner on bare metal Linux. This gives you **unlimited CI/CD minutes** for your own hardware costs.

**Benefits:**
- ✅ Unlimited minutes (no GitHub quota)
- ✅ Faster builds (with good hardware)
- ✅ Custom environment control
- ✅ Cost-effective for high usage

**Trade-offs:**
- ⚠️ Security responsibility (isolation, updates)
- ⚠️ Hardware maintenance
- ⚠️ Network/power reliability

---

## 📋 Prerequisites

### Hardware Requirements

**Minimum:**
- CPU: 2 cores (4+ recommended)
- RAM: 4 GB (8+ GB recommended)
- Disk: 40 GB free (100+ GB for cargo cache)
- Network: Stable connection, 10+ Mbps

**Recommended for Rust builds:**
- CPU: 8 cores (Ryzen 5 or Intel i5+)
- RAM: 16 GB
- Disk: 256 GB SSD (NVMe for best cargo build performance)
- Network: 100+ Mbps

**Example hardware:**
- Old desktop/workstation
- Intel NUC (~$300-500)
- System76 Thelio Mira (~$900+)
- Used server (Dell PowerEdge R730, ~$300-600)

### Operating System

**Supported Linux distributions:**
- ✅ **Ubuntu 20.04/22.04 LTS** (recommended, best tested)
- ✅ Debian 11/12
- ✅ Fedora 38+
- ✅ RHEL/Rocky Linux 8/9
- ✅ Arch Linux (if you're adventurous)

**This guide uses Ubuntu 22.04 LTS** - adjust package names for other distros.

---

## 🔧 Initial System Setup

### 1. Install and Update System

```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Install essential tools
sudo apt install -y \
    curl \
    wget \
    git \
    build-essential \
    libssl-dev \
    pkg-config \
    jq \
    ca-certificates \
    gnupg \
    lsb-release
```

### 2. Create Dedicated User

**IMPORTANT:** Never run the runner as root!

```bash
# Create dedicated user for GitHub Actions
sudo useradd -m -s /bin/bash github-runner

# Add to docker group (if using Docker)
sudo usermod -aG docker github-runner

# Switch to runner user
sudo su - github-runner
```

### 3. Install Rust Toolchain

```bash
# Install rustup (as github-runner user)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Load cargo env
source "$HOME/.cargo/env"

# Install stable + nightly toolchains
rustup toolchain install stable
rustup toolchain install nightly
rustup default stable

# Install components
rustup component add rustfmt clippy

# Verify installation
cargo --version
rustc --version
```

### 4. Install Docker (Optional but Recommended)

Docker provides isolation for jobs that need it.

```bash
# As root/sudo user (not github-runner)
exit  # Exit from github-runner user

# Add Docker's official GPG key
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
    sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg

# Set up repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
  https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker Engine
sudo apt update
sudo apt install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin

# Enable and start Docker
sudo systemctl enable docker
sudo systemctl start docker

# Add github-runner to docker group
sudo usermod -aG docker github-runner

# Verify Docker works
sudo docker run hello-world
```

---

## 🚀 GitHub Actions Runner Installation

### 1. Download Runner

Switch back to `github-runner` user:

```bash
sudo su - github-runner
cd ~

# Create runner directory
mkdir -p actions-runner && cd actions-runner

# Download latest runner (check https://github.com/actions/runner/releases for latest version)
RUNNER_VERSION="2.311.0"
curl -o actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz -L \
    https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz

# Verify checksum (optional but recommended)
echo "29fc8cf2dab4c195bb147384e7e2c94cfd4d4022c793b346a6175435265aa278  actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz" | shasum -a 256 -c

# Extract
tar xzf ./actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz
```

### 2. Get Registration Token

You need a token to register the runner with your repository.

**Option A: Via GitHub Web UI** (easiest)
1. Go to: https://github.com/prasincs/universal-blockchain-decoder/settings/actions/runners/new
2. Copy the token from the "Configure" section

**Option B: Via GitHub CLI** (automated)
```bash
# Install gh CLI (as root)
exit  # Exit github-runner user
sudo apt install gh -y

# Authenticate
gh auth login

# Get registration token
TOKEN=$(gh api \
    --method POST \
    -H "Accept: application/vnd.github+json" \
    /repos/prasincs/universal-blockchain-decoder/actions/runners/registration-token \
    | jq -r .token)

echo $TOKEN
```

### 3. Configure Runner

```bash
# Switch back to github-runner
sudo su - github-runner
cd ~/actions-runner

# Configure runner
./config.sh \
    --url https://github.com/prasincs/universal-blockchain-decoder \
    --token YOUR_TOKEN_HERE \
    --name "bare-metal-linux-01" \
    --labels "self-hosted,linux,bare-metal,rust" \
    --work _work

# When prompted:
# - Runner group: Press Enter (Default)
# - Work folder: Press Enter (_work)
```

**Important configuration notes:**
- `--name`: Unique name for this runner (useful if you have multiple)
- `--labels`: Tags to target specific jobs (more on this below)
- `--work`: Working directory for jobs (default is fine)

### 4. Install as System Service

This makes the runner start automatically on boot.

```bash
# As root (exit github-runner user first)
exit

# Install service (as root)
cd /home/github-runner/actions-runner
sudo ./svc.sh install github-runner

# Start service
sudo ./svc.sh start

# Check status
sudo ./svc.sh status

# Enable auto-start on boot
sudo systemctl enable actions.runner.prasincs-universal-blockchain-decoder.bare-metal-linux-01.service
```

### 5. Verify Runner is Connected

1. Go to: https://github.com/prasincs/universal-blockchain-decoder/settings/actions/runners
2. You should see your runner listed as "Idle" (green)

---

## 🔒 Security Hardening

### 1. Firewall Configuration

```bash
# Enable UFW firewall
sudo ufw enable

# Allow SSH (adjust port if non-standard)
sudo ufw allow 22/tcp

# Allow outbound HTTPS (GitHub API)
sudo ufw allow out 443/tcp

# Deny all inbound by default (except SSH)
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Check status
sudo ufw status verbose
```

### 2. Disable Root SSH (if not already)

```bash
# Edit SSH config
sudo nano /etc/ssh/sshd_config

# Set these values:
# PermitRootLogin no
# PasswordAuthentication no
# PubkeyAuthentication yes

# Restart SSH
sudo systemctl restart sshd
```

### 3. Set Up Automatic Security Updates

```bash
# Install unattended-upgrades
sudo apt install -y unattended-upgrades

# Enable automatic security updates
sudo dpkg-reconfigure -plow unattended-upgrades

# Configure (optional)
sudo nano /etc/apt/apt.conf.d/50unattended-upgrades
```

### 4. Isolate Runner Workspace

The runner's `_work` directory should have restricted permissions:

```bash
# As github-runner user
cd ~/actions-runner
chmod 700 _work

# Set up ephemeral workspace cleanup (optional)
# Add to crontab: clean _work every night
crontab -e

# Add this line:
0 2 * * * find /home/github-runner/actions-runner/_work -type d -mtime +7 -exec rm -rf {} +
```

### 5. Limit Runner to Specific Branches

**CRITICAL for security:** Only allow runner on trusted branches, NOT pull requests from external contributors.

In your workflow files, use:

```yaml
# SAFE: Only runs on your repository's code
on:
  push:
    branches: [main, 'claude/**']

jobs:
  test:
    runs-on: [self-hosted, linux, bare-metal]
```

```yaml
# UNSAFE: External PRs can run arbitrary code on your machine!
on:
  pull_request:  # ⚠️ DANGER

jobs:
  test:
    runs-on: [self-hosted, linux]  # ❌ Don't use self-hosted for PRs
```

**Best practice:** Use GitHub-hosted for pull requests, self-hosted for pushes.

---

## 🎯 Using Your Self-Hosted Runner

### Update Workflow Files

Edit `.github/workflows/test.yml`:

```yaml
name: Test Suite

on:
  push:
    branches: [main, master, 'claude/**']
    paths:
      - '**.rs'
      - '**/Cargo.toml'
      - '**/Cargo.lock'

jobs:
  unit-tests:
    name: Unit Tests
    runs-on: [self-hosted, linux, bare-metal]  # ← Use your runner

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          submodules: recursive

      - name: Run tests
        run: cargo test --lib --all --verbose

  # Keep expensive jobs on GitHub-hosted
  security-audit:
    name: Security Audit
    runs-on: ubuntu-latest  # ← Still use GitHub for security audit
    if: github.event_name == 'pull_request'

    steps:
      - uses: actions/checkout@v4
      - run: cargo audit
```

### Runner Label Strategy

You can target specific runners using labels:

```yaml
# Use any Linux runner (GitHub or self-hosted)
runs-on: linux

# Require self-hosted
runs-on: [self-hosted, linux]

# Require specific runner
runs-on: [self-hosted, linux, bare-metal]

# Use GitHub-hosted (default)
runs-on: ubuntu-latest
```

**Recommended approach:**

```yaml
jobs:
  # Fast jobs on self-hosted (unlimited)
  unit-tests:
    runs-on: [self-hosted, linux]

  clippy:
    runs-on: [self-hosted, linux]

  # Expensive/infrequent jobs on GitHub-hosted
  coverage:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'

  # Security-critical on GitHub-hosted (isolated)
  security-audit:
    runs-on: ubuntu-latest
```

---

## 📊 Monitoring & Maintenance

### 1. Check Runner Status

```bash
# Check service status
sudo systemctl status actions.runner.*.service

# View runner logs
sudo journalctl -u actions.runner.*.service -f

# Check runner processes
ps aux | grep Runner.Listener
```

### 2. Monitor Disk Space

Cargo builds can consume significant disk space:

```bash
# Check disk usage
df -h

# Check cargo cache size
du -sh ~/.cargo

# Clean old cargo cache
cargo cache --autoclean
# OR manually:
rm -rf ~/.cargo/registry/cache
rm -rf ~/.cargo/git/db

# Clean build artifacts
cd ~/actions-runner/_work
find . -type d -name "target" -exec rm -rf {} +
```

### 3. Update Runner

GitHub periodically releases runner updates:

```bash
# Stop runner
sudo ./svc.sh stop

# Download new version
cd /home/github-runner
RUNNER_VERSION="2.312.0"  # Check latest version
wget https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz

# Backup old runner
cd actions-runner
cp -r . ../actions-runner.backup

# Extract new version
tar xzf ../actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz

# Restart service
sudo ./svc.sh start
```

### 4. Automated Monitoring Script

Create `/home/github-runner/monitor.sh`:

```bash
#!/bin/bash
# Runner health check script

# Check if runner service is running
if ! systemctl is-active --quiet actions.runner.*.service; then
    echo "❌ Runner service is down!"
    sudo systemctl restart actions.runner.*.service
fi

# Check disk space (warn if < 10GB free)
FREE_SPACE=$(df /home | tail -1 | awk '{print $4}')
if [ "$FREE_SPACE" -lt 10485760 ]; then  # 10GB in KB
    echo "⚠️  Low disk space: $(df -h /home | tail -1)"
    # Clean cargo cache
    cargo cache --autoclean
fi

# Check memory usage
MEM_USED=$(free | grep Mem | awk '{print int($3/$2 * 100)}')
if [ "$MEM_USED" -gt 90 ]; then
    echo "⚠️  High memory usage: ${MEM_USED}%"
fi

echo "✅ Runner health check passed"
```

Add to crontab:

```bash
chmod +x /home/github-runner/monitor.sh
crontab -e

# Add: Run every 15 minutes
*/15 * * * * /home/github-runner/monitor.sh >> /home/github-runner/monitor.log 2>&1
```

---

## 🚨 Troubleshooting

### Runner Won't Start

```bash
# Check service status
sudo systemctl status actions.runner.*.service

# View detailed logs
sudo journalctl -u actions.runner.*.service -n 100

# Common issues:
# 1. Token expired - Re-register runner
# 2. Network issues - Check firewall, DNS
# 3. Disk full - Clean cargo cache
```

### Jobs Not Picking Up

1. Check runner is online: https://github.com/YOUR_REPO/settings/actions/runners
2. Verify workflow uses correct labels:
   ```yaml
   runs-on: [self-hosted, linux]  # Must match runner labels
   ```
3. Check runner logs:
   ```bash
   sudo journalctl -u actions.runner.*.service -f
   ```

### High Disk Usage

```bash
# Find large directories
du -sh ~/actions-runner/_work/* | sort -h

# Clean cargo cache
cargo cache --autoclean

# Clean old build artifacts
find ~/actions-runner/_work -name "target" -type d -mtime +7 -exec rm -rf {} +

# Clean Docker images (if using Docker)
docker system prune -a --volumes
```

### Jobs Hanging/Timing Out

```bash
# Check system resources
htop

# Check for zombie processes
ps aux | grep defunct

# Restart runner
sudo ./svc.sh restart
```

---

## 💰 Cost Analysis

### Bare Metal Linux Runner Costs

**Hardware options:**

| Hardware | Cost | Performance | Notes |
|----------|------|-------------|-------|
| Old desktop (repurposed) | $0 | ⭐⭐⭐ | Free, but uses power |
| Intel NUC 11 (i5, 16GB) | $500 | ⭐⭐⭐⭐ | Compact, efficient |
| System76 Thelio Mira | $900+ | ⭐⭐⭐⭐⭐ | High performance |
| Used Dell R730 server | $300-600 | ⭐⭐⭐⭐⭐ | Loud, power-hungry |

**Operating costs:**

```
Monthly costs (assuming 24/7 operation):
- Power: $5-30/month (depends on hardware, electricity rates)
- Internet: $0 (use existing connection)
- Maintenance: $0 (DIY)

Total: $5-30/month for UNLIMITED minutes
```

**Break-even analysis:**

```
GitHub Pro: 3000 minutes/month included
Over 3000 minutes: $0.008/minute

Self-hosted break-even:
$500 hardware ÷ $0.008/minute = 62,500 minutes
62,500 ÷ 3000 = ~21 months of maxing out GitHub minutes

If you use 6000+ min/month consistently:
- GitHub cost: $24/month (3000 extra minutes)
- Self-hosted: $20/month (hardware amortized + power)
- Break-even: ~2 years
```

---

## 🎓 Next Steps

1. **Test your runner:**
   ```bash
   # Trigger a workflow manually
   gh workflow run test.yml
   ```

2. **Monitor performance:**
   - Check runner dashboard in GitHub
   - Monitor system resources
   - Track build times vs GitHub-hosted

3. **Optimize workflows:**
   - Use self-hosted for frequent jobs
   - Keep GitHub-hosted for PRs and security audits
   - Cache cargo dependencies aggressively

4. **Scale up (optional):**
   - Add more runners for parallelism
   - Set up runner pools
   - Implement auto-scaling with Kubernetes

---

## 📚 References

- [GitHub Actions Self-Hosted Runners](https://docs.github.com/en/actions/hosting-your-own-runners)
- [Runner Security Best Practices](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions)
- [Rust CI/CD Best Practices](https://matklad.github.io/2021/09/04/fast-rust-builds.html)

---

## ✅ Summary Checklist

- [ ] System updated and hardened
- [ ] Dedicated `github-runner` user created
- [ ] Rust toolchain installed
- [ ] Docker installed (optional)
- [ ] Runner downloaded and configured
- [ ] Service installed and running
- [ ] Firewall configured
- [ ] Runner visible in GitHub settings (Idle/Online)
- [ ] Workflow updated to use `[self-hosted, linux]`
- [ ] Test job runs successfully
- [ ] Monitoring script set up
- [ ] Backup plan for hardware failure

---

**Questions?** Check the troubleshooting section or open an issue in the repository.

**Security concern?** Review the Security Hardening section and never use self-hosted runners for public pull requests.
