# anspm - AkaruiNeko's Package Manager

![Rust](https://img.shields.io/badge/Rust-1.88+-informational?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

A modern package manager written in Rust, designed for simplicity and performance.

# **What does `anspm` stand for?**
- **AkaruiNeko's Simple Package Manager** – the original meaning 😺
- **Another Neat Simple Package Manager** – because simplicity matters
- **Ain’t No Stupid Package Manager** – because it Just Works™

## Features

- Fast dependency resolution
- GPG-verified repositories
- Supports custom `.pkg` package format
- Cross-platform (Linux/macOS)
- Powerful search functionality

## Installation

### From Source

```bash
# Clone repository
git clone https://github.com/BrightCat14/anspm.git
cd anspm

# Build (requires Rust 1.88+)
cargo build --release

# Install system-wide
sudo cp target/release/anspm /usr/local/bin/
```

### Pre-built Binaries

Download from [Releases](https://github.com/BrightCat14/anspm/releases) page.

## Usage

```bash
# Search for packages
anspm search <query>

# Install a package
anspm install <package>

# List installed packages
anspm list

# Update package database
anspm update

# and etc
```

## Package Format

anspm uses custom `.pkg` bundles containing:
```
package.pkg/
├── meta.toml     # Package metadata
└── usr/          # Files to install
# you can add your files here
```

## Development

### Build Dependencies

- Rust 1.88+
- OpenSSL/LibreSSL
- GPG (for repository signing)

### Cross-compilation

```bash
# For macOS (from Linux)
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

## Contributing

Pull requests are welcome! Please follow:
1. Rust coding conventions
2. Commit message guidelines
3. Test coverage requirements

## License

MIT © 2025 George Kulikov
