# Ellex CI/CD Workflows

This directory contains comprehensive CI/CD workflows for the Ellex natural language programming environment.

## Overview

The Ellex project uses GitHub Actions for continuous integration, testing, and automated releases. The workflows are designed to ensure code quality, run comprehensive tests, and create cross-platform binary releases.

## Workflows

### 🔄 `ci.yml` - Continuous Integration
**Triggers:** Push/PR to `main` or `develop` branches

**Jobs:**
- **Check**: Basic code validation with `cargo check`
- **Format**: Enforces Rust formatting with `rustfmt`
- **Clippy**: Linting with Clippy (treats warnings as errors)
- **Test**: Cross-platform test suite (Ubuntu, Windows, macOS)
- **Coverage**: Code coverage analysis with `cargo-llvm-cov` and Codecov integration
- **Security Audit**: Dependency vulnerability scanning with `cargo-audit`
- **Build Test**: Verify binary builds and runs on all platforms

### 🚀 `release.yml` - Cross-Platform Releases
**Triggers:** Git tags (`v*.*.*`), PRs, manual dispatch

**Build Matrix:**
- **Linux**: x64, ARM64 (GNU libc)
- **macOS**: x64 (Intel), ARM64 (Apple Silicon)
- **Windows**: x64, ARM64 (MSVC)

**Artifacts:**
- Compressed binaries (`tar.gz` for Unix, `.zip` for Windows)
- Automatic GitHub release creation with download links
- Release notes with installation instructions

**Binary Name:** `el` (configured in `ellex_cli/Cargo.toml`)

### ⚡ `benchmark.yml` - Performance Testing
**Triggers:** Push to `main`, PRs, daily schedule (2 AM UTC), manual dispatch

**Features:**
- Transpiler performance benchmarks vs TSC, SWC
- Cargo benchmark suite execution
- Performance regression detection
- Automated PR comments with benchmark results
- Daily performance monitoring

### 📚 `docs.yml` - Documentation
**Triggers:** Push/PR to `main`, manual dispatch

**Features:**
- Rust documentation generation with `cargo doc`
- GitHub Pages deployment
- mdBook documentation compilation
- Missing documentation checks

### 🔄 `dependabot-auto-merge.yml` - Dependency Management
**Triggers:** Dependabot PRs

**Features:**
- Auto-merge minor and patch updates
- CI validation before merge
- Major version updates require manual review

## Configuration Files

### `.github/dependabot.yml`
Automated dependency updates for:
- **Rust** dependencies (weekly, Mondays)
- **GitHub Actions** (weekly, Mondays)
- **Node.js** frontend dependencies (weekly, Tuesdays)
- **Elixir** backend dependencies (weekly, Wednesdays)

### `.pre-commit-config.yaml`
Pre-commit hooks for code quality:
- `cargo fmt` - Rust formatting
- `cargo clippy` - Rust linting
- `cargo test` - Test execution (on push)
- Python formatting (Black, isort) for benchmark scripts
- YAML, TOML validation

## Binary Configuration

The main CLI binary is configured to build as `el`:

```toml
# crates/ellex_cli/Cargo.toml
[[bin]]
name = "el"
path = "src/main.rs"
```

## Release Process

### Automatic Releases
1. Create and push a git tag: `git tag v1.0.0 && git push origin v1.0.0`
2. GitHub Actions automatically:
   - Builds cross-platform binaries
   - Creates GitHub release
   - Uploads binary artifacts
   - Generates release notes

### Manual Testing
```bash
# Build locally
make build

# Run tests
make test

# Install locally
make install

# Test binary
el --version
el --help
```

## Makefile Integration

The root `Makefile` has been updated to support the new binary name:

```bash
make build     # Build el binary
make install   # Install el binary locally
make release   # Build optimized release binary
make test      # Run all tests
make fmt       # Format code
make clippy    # Run linting
make check     # Check without building
make bench     # Run benchmarks
```

## Environment Requirements

### GitHub Secrets
- `GITHUB_TOKEN`: Automatically provided
- `CODECOV_TOKEN`: Optional, for code coverage reports

### Dependencies
All required tools are automatically installed in CI:
- Rust toolchain with cross-compilation targets
- Node.js (for TypeScript benchmarks)
- Python (for analysis scripts)
- System dependencies for cross-compilation

## Development Workflow

### Local Development
1. `make setup` - Initialize development environment
2. `make dev` - Start interactive REPL
3. `make test` - Run test suite
4. `make fmt && make clippy` - Format and lint code

### Pull Request Process
1. Create feature branch
2. Make changes
3. Run `make fmt clippy test` locally
4. Push branch and create PR
5. CI automatically runs full test suite
6. Benchmarks compare performance against main branch
7. Merge after CI passes and review approval

### Release Process
1. Update version in `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v1.0.0`
4. Push tag: `git push origin v1.0.0`
5. GitHub Actions creates release automatically

## Monitoring and Alerts

- **Daily benchmarks** track performance regressions
- **Security audits** catch vulnerable dependencies
- **Code coverage** reports track test completeness
- **Cross-platform builds** ensure compatibility

## Next Steps

1. **WebAssembly Builds**: Add WASM target to release matrix
2. **Docker Images**: Create containerized releases
3. **Package Managers**: Add Homebrew, Chocolatey, APT packages
4. **Performance Monitoring**: Continuous benchmark result tracking
5. **Integration Tests**: End-to-end workflow validation

---

*This CI/CD setup provides enterprise-grade automation for the Ellex programming language, ensuring quality, compatibility, and reliable releases across all supported platforms.*