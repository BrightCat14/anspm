# anspm - AkaruiNeko's Package Manager

![Rust](https://img.shields.io/badge/Rust-1.88+-informational?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

A modern package manager written in Rust, designed for simplicity and performance.

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

# Build (requires Rust 1.70+)
cargo build --release

# Install system-wide
sudo cp target/release/anspm /usr/local/bin/
