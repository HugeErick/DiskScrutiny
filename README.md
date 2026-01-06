# DiskScrutiny

[![Rust](https://img.shields.io/badge/Rust-1.8%2B-orange)](https://rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

## Description

DiskScrutiny is a high-performance disk space analyzer built in Rust using the imgui-rs framework. Inspired by tools like WizTree, it helps users instantly identify which folders and files are consuming the most space on their drives.

## Table of Contents

- [Installation](#Installation)
  - [Windows](#Windows)
  - [Linux](#Linux)
    - [1. Install System Dependencies](#1-Install-System-Dependencies)
    - [2. Clone and Build](#2-Clone-and-Build)
- [License](#License)
- [Contact](#Contact)

## Installation

### Windows

**Official releases for Windows are coming soon.** For now, Windows users can follow the manual compilation steps below if they have the Rust toolchain installed. Note that MFT scanning requires running the terminal as **Administrator**.

### Linux

Currently, Linux users can install DiskScrutiny by compiling from source using Cargo.

#### 1. Install System Dependencies

Since DiskScrutiny uses a graphical interface, you need the following development libraries.

**For Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install build-essential python3 libx11-dev libxft-dev libxext-dev

```

**For Fedora:**

```bash
sudo dnf install @development-tools libX11-devel libXft-devel libXext-devel

```

#### 2. Clone and Build

```bash
# Clone the repository
git clone https://github.com/HugeErick/DiskScrutiny.git
cd DiskScrutiny

# Build and run in release mode for best performance
cargo run --release

```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

Erick Gonzalez Parada - <erick.parada101@gmail.com>

Project Link: [https://github.com/HugeErick/DiskScrutiny](https://github.com/HugeErick/DiskScrutiny)

