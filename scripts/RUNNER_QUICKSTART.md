# Self-Hosted Runner - Quick Start

## 🚀 One-Command Setup

```bash
# 1. Get registration token from:
#    https://github.com/prasincs/universal-blockchain-decoder/settings/actions/runners/new

# 2. Run setup script (as root)
sudo ./scripts/setup-runner.sh YOUR_TOKEN_HERE

# 3. Verify runner online:
#    https://github.com/prasincs/universal-blockchain-decoder/settings/actions/runners
```

That's it! Your runner is now ready to accept jobs.

---

## 📋 What the Script Does

1. ✅ Updates system packages
2. ✅ Installs dependencies (git, build-essential, Docker, etc.)
3. ✅ Creates `github-runner` user
4. ✅ Installs Rust toolchain (stable + nightly)
5. ✅ Downloads GitHub Actions runner
6. ✅ Configures runner with your token
7. ✅ Installs as system service (auto-start on boot)
8. ✅ Sets up firewall (UFW)
9. ✅ Creates health monitoring script
10. ✅ Starts runner service

**Time required:** 5-10 minutes (depending on internet speed)

---

## 🎯 Using Your Runner

Update `.github/workflows/test.yml`:

```yaml
jobs:
  unit-tests:
    name: Unit Tests
    runs-on: [self-hosted, linux, bare-metal, rust]  # ← Use your runner

  # Keep PRs on GitHub-hosted (security)
  pr-tests:
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest  # ← GitHub-hosted for external PRs
```

---

## 🔧 Service Management

```bash
# Start runner
sudo /home/github-runner/actions-runner/svc.sh start

# Stop runner
sudo /home/github-runner/actions-runner/svc.sh stop

# Restart runner
sudo /home/github-runner/actions-runner/svc.sh restart

# Check status
sudo /home/github-runner/actions-runner/svc.sh status

# View logs
sudo journalctl -u actions.runner.*.service -f
```

---

## 📊 Monitoring

```bash
# Check runner health
/home/github-runner/monitor.sh

# View monitoring logs
tail -f /home/github-runner/monitor.log

# Check disk space
df -h

# Check cargo cache size
du -sh ~/.cargo

# Clean cargo cache
cargo cache --autoclean
```

---

## 🚨 Troubleshooting

### Runner Won't Start
```bash
# Check logs
sudo journalctl -u actions.runner.*.service -n 100

# Restart service
sudo systemctl restart actions.runner.*.service
```

### Jobs Not Running
1. Verify runner online in GitHub settings
2. Check workflow uses correct labels: `[self-hosted, linux]`
3. Check logs for errors

### High Disk Usage
```bash
# Clean cargo cache
cargo cache --autoclean

# Remove old build artifacts
find /home/github-runner/actions-runner/_work -name "target" -type d -mtime +7 -exec rm -rf {} +

# Clean Docker (if using)
docker system prune -a
```

---

## 🔒 Security Notes

**IMPORTANT:**
- ✅ Only use for `push` events (your code)
- ❌ **Never** use for external `pull_request` events
- ✅ Keep system updated (`sudo apt update && sudo apt upgrade`)
- ✅ Monitor logs regularly

**Unsafe workflow example:**
```yaml
on:
  pull_request:  # ⚠️ DANGER - external code

jobs:
  test:
    runs-on: self-hosted  # ❌ Don't do this!
```

**Safe workflow example:**
```yaml
on:
  push:
    branches: [main, 'claude/**']  # ✅ Only your branches

jobs:
  test:
    runs-on: self-hosted  # ✅ Safe
```

---

## 📖 Full Documentation

See `docs/SELF_HOSTED_RUNNER_LINUX.md` for:
- Hardware requirements
- Manual setup instructions
- Advanced configuration
- Security hardening
- Cost analysis
- Detailed troubleshooting

---

## ✅ Quick Checklist

After setup completes:

- [ ] Runner shows "Idle" in GitHub settings
- [ ] Test workflow runs successfully
- [ ] Monitoring script works (`/home/github-runner/monitor.sh`)
- [ ] Service auto-starts on reboot
- [ ] Firewall configured
- [ ] Disk space monitored

---

## 💰 Minute Savings Estimate

**Before (GitHub-hosted only):**
- ~800-1200 min/month after optimization
- 3000 min/month limit

**After (self-hosted Linux):**
- **Unlimited minutes** for Linux jobs
- Only use GitHub minutes for:
  - macOS builds (if needed)
  - External PR validation
  - Security audits

**Estimated savings:** ~90%+ of GitHub Actions minutes

---

## 🎓 Next Steps

1. **Test your runner:**
   ```bash
   gh workflow run test.yml
   ```

2. **Monitor first few runs:**
   ```bash
   sudo journalctl -u actions.runner.*.service -f
   ```

3. **Optimize workflows:**
   - Move frequent jobs to self-hosted
   - Keep security-critical jobs on GitHub-hosted
   - Use labels to target specific runners

4. **Scale (optional):**
   - Add more runners for parallelism
   - Set up runner auto-scaling
   - Deploy on multiple machines

---

**Questions?** Check full docs: `docs/SELF_HOSTED_RUNNER_LINUX.md`
