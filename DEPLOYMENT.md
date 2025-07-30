# Ellex Deployment Guide

This document provides comprehensive instructions for deploying and distributing the Ellex natural language programming environment.

## 🚀 Quick Start

### Binary Releases

Download the latest release for your platform from [GitHub Releases](https://github.com/ellex-lang/ellex/releases):

```bash
# Linux x64
curl -L https://github.com/ellex-lang/ellex/releases/latest/download/el-linux-x64.tar.gz | tar -xz
chmod +x el
sudo mv el /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/ellex-lang/ellex/releases/latest/download/el-macos-x64.tar.gz | tar -xz
chmod +x el
sudo mv el /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/ellex-lang/ellex/releases/latest/download/el-macos-arm64.tar.gz | tar -xz
chmod +x el
sudo mv el /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/ellex-lang/ellex/releases/latest/download/el-windows-x64.exe.zip" -OutFile "el.zip"
Expand-Archive -Path "el.zip" -DestinationPath "."
# Add to PATH manually or move to a directory in PATH
```

### Verify Installation

```bash
el --version
el --help
```

## 📦 Installation Methods

### 1. GitHub Releases (Recommended)

Pre-built binaries for all major platforms are automatically generated and published to GitHub Releases on every tagged version.

**Supported Platforms:**
- Linux x64 (`x86_64-unknown-linux-gnu`)
- Linux ARM64 (`aarch64-unknown-linux-gnu`)
- macOS x64 (`x86_64-apple-darwin`)
- macOS ARM64 (`aarch64-apple-darwin`) - Apple Silicon
- Windows x64 (`x86_64-pc-windows-msvc`)
- Windows ARM64 (`aarch64-pc-windows-msvc`)

### 2. Build from Source

**Prerequisites:**
- Rust 1.75+ (`rustup install stable`)
- Git

**Build Process:**
```bash
git clone https://github.com/ellex-lang/ellex.git
cd ellex
make setup    # Install dependencies
make build    # Build release binary
make install  # Install to ~/.cargo/bin/el
```

**Development Build:**
```bash
cd crates
cargo build --release --bin el
./target/release/el --version
```

### 3. Package Managers (Future)

Package manager support is planned for:
- **Homebrew** (macOS/Linux): `brew install ellex`
- **Chocolatey** (Windows): `choco install ellex`
- **APT** (Ubuntu/Debian): `apt install ellex`
- **AUR** (Arch Linux): `yay -S ellex`
- **Cargo**: `cargo install ellex`

## 🔧 CI/CD Pipeline

### Automated Release Process

The project uses GitHub Actions for automated building, testing, and releasing:

1. **Tag Creation**: `git tag v1.0.0 && git push origin v1.0.0`
2. **Automated Build**: Cross-platform binaries built in parallel
3. **Testing**: Full test suite runs on all platforms
4. **Release Creation**: GitHub release with download links
5. **Artifact Upload**: Compressed binaries for all platforms

### Release Workflow

```yaml
# Trigger release
git tag v1.2.3
git push origin v1.2.3

# GitHub Actions automatically:
# 1. Runs full test suite
# 2. Builds cross-platform binaries
# 3. Creates GitHub release
# 4. Uploads artifacts
# 5. Generates release notes
```

### Quality Gates

Before release, all builds must pass:
- ✅ Unit tests (`cargo test`)
- ✅ Integration tests
- ✅ Linting (`cargo clippy`)
- ✅ Formatting (`cargo fmt --check`)
- ✅ Security audit (`cargo audit`)
- ✅ Cross-platform compatibility
- ✅ Performance benchmarks

## 🏗️ Build Configuration

### Binary Configuration

The main CLI is configured as `el` in `crates/ellex_cli/Cargo.toml`:

```toml
[[bin]]
name = "el"
path = "src/main.rs"
```

### Cross-Compilation Setup

GitHub Actions uses Rust's built-in cross-compilation:

```yaml
strategy:
  matrix:
    include:
      - target: x86_64-unknown-linux-gnu
        os: ubuntu-latest
      - target: aarch64-apple-darwin
        os: macos-latest
      - target: x86_64-pc-windows-msvc
        os: windows-latest
```

### Optimization Flags

Release builds use these optimization settings:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## 🐳 Docker Deployment

### Docker Images (Planned)

Future Docker image variants:

```dockerfile
# Runtime image
FROM alpine:3.19
COPY --from=builder /usr/local/bin/el /usr/local/bin/el
ENTRYPOINT ["el"]

# Development image
FROM rust:1.75-alpine
# Full development environment
```

### Container Registry

Planned container registries:
- **GitHub Container Registry**: `ghcr.io/ellex-lang/ellex`
- **Docker Hub**: `ellexlang/ellex`

## 🌐 Web Deployment

### Playground Server

The Ellex binary includes a web server for the playground:

```bash
el serve --port 3000
# Starts web playground on http://localhost:3000
```

### WebAssembly Build

For web deployment, Ellex can be compiled to WebAssembly:

```bash
# Future functionality
el transpile playground.ellex --target wasm
# Generates playground.wasm + loader.js
```

## 📊 Monitoring and Analytics

### Performance Tracking

Automated benchmarks track:
- **Compilation Speed**: Lines per second
- **Memory Usage**: Peak and average
- **Binary Size**: Release binary sizes
- **Startup Time**: Cold start performance

### Usage Analytics (Optional)

For hosted deployments, optional analytics can track:
- Command usage patterns
- Error frequencies
- Performance metrics
- Feature adoption

## 🔒 Security Considerations

### Binary Signing

Future releases will include:
- **Code Signing**: Signed binaries for Windows/macOS
- **Checksums**: SHA256 hashes for all releases
- **GPG Signatures**: Detached signatures for verification

### Supply Chain Security

- **Dependency Audits**: Automated vulnerability scanning
- **Reproducible Builds**: Deterministic build process
- **SBOM Generation**: Software Bill of Materials
- **Security Advisories**: CVE monitoring and patching

## 🚀 Deployment Environments

### Development

```bash
make dev    # Start REPL for development
el repl     # Interactive development environment
```

### Testing

```bash
make test   # Run full test suite
el run test.ellex  # Run specific test file
```

### Production

```bash
# Server deployment
el serve --port 8080 --host 0.0.0.0

# Batch processing
el run production-script.ellex

# TUI monitoring
el tui  # Real-time metrics dashboard
```

## 📋 Deployment Checklist

### Pre-Release

- [ ] Update version numbers in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Run full test suite: `make test`
- [ ] Verify cross-platform builds locally
- [ ] Update documentation
- [ ] Review security audit results

### Release Process

- [ ] Create and push version tag
- [ ] Verify GitHub Actions build success
- [ ] Test downloaded binaries on each platform
- [ ] Update installation documentation
- [ ] Announce release

### Post-Release

- [ ] Monitor for issues and bug reports
- [ ] Update package manager formulas
- [ ] Collect performance metrics
- [ ] Plan next release features

## 🆘 Troubleshooting

### Common Issues

**Build Failures:**
```bash
# Clear cache and rebuild
cargo clean
make build
```

**Permission Issues (Linux/macOS):**
```bash
chmod +x el
sudo chown root:root /usr/local/bin/el
```

**Windows Execution Policy:**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Debug Information

```bash
el --version           # Version info
RUST_LOG=debug el repl # Verbose logging
```

### Getting Help

- **Issues**: [GitHub Issues](https://github.com/ellex-lang/ellex/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ellex-lang/ellex/discussions)
- **Documentation**: [User Guide](https://ellex-lang.org/docs)

---

*This deployment guide ensures reliable, secure, and efficient distribution of the Ellex programming environment across all supported platforms.*