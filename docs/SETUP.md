# PWE Karaoke - Setup Guide

This document provides a step-by-step guide to set up the development environment for PWE Karaoke.

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Installation Steps](#installation-steps)
3. [Verification](#verification)
4. [Troubleshooting](#troubleshooting)

## System Requirements

### Minimum Requirements

- **OS**: Linux (Ubuntu 20.04+, Fedora 35+), macOS (11+), Windows 10/11
- **RAM**: 4GB (8GB recommended for Spleeter processing)
- **Storage**: 2GB free space (plus space for music library and models)
- **CPU**: Multi-core processor recommended for real-time audio processing

### Software Requirements

- Rust 1.70+ (latest stable recommended)
- Python 3.8, 3.9, 3.10, or 3.11 (3.11 recommended)
- FFmpeg 4.0+
- Git

## Installation Steps

### Step 1: Install System Dependencies

#### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    libgtk-3-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libspeechd-dev \
    libxkbcommon-dev \
    python3 \
    python3-pip \
    python3-dev \
    python3-venv \
    ffmpeg
```

#### Fedora/RHEL

```bash
sudo dnf install -y \
    gcc \
    gcc-c++ \
    make \
    curl \
    git \
    pkg-config \
    openssl-devel \
    alsa-lib-devel \
    gtk3-devel \
    python3 \
    python3-pip \
    python3-devel \
    ffmpeg
```

#### macOS

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install python@3.11 pkg-config ffmpeg
```

#### Windows

1. Install [Visual Studio Build Tools 2019+](https://visualstudio.microsoft.com/downloads/)
   - Select "Desktop development with C++"
2. Install [Python 3.11](https://www.python.org/downloads/)
   - Check "Add Python to PATH" during installation
3. Install [FFmpeg](https://ffmpeg.org/download.html)
   - Download, extract, and add to system PATH
4. Install [Git for Windows](https://git-scm.com/download/win)

### Step 2: Install Rust

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts (choose option 1 for default installation)

# Reload your shell configuration
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Step 3: Clone the Repository

```bash
git clone <repository-url>
cd pwe
```

### Step 4: Set Up Python Environment

```bash
# Create a virtual environment
python3 -m venv venv

# Activate the virtual environment
# On Linux/macOS:
source venv/bin/activate
# On Windows:
# venv\Scripts\activate

# Upgrade pip
pip install --upgrade pip

# Install Python dependencies
pip install -r requirements.txt

# Verify Spleeter installation
spleeter --version
```

### Step 5: Configure Environment (Optional)

```bash
# Copy example environment file
cp .env.example .env

# Edit .env if needed
# nano .env
```

### Step 6: Build the Project

```bash
# Make sure Python virtual environment is activated
source venv/bin/activate  # Skip on Windows after already activated

# Build in development mode
cargo build

# This will take several minutes on first build
# PyO3 will compile Python bindings
# All Rust dependencies will be downloaded and compiled
```

## Verification

### Verify Rust Setup

```bash
cargo --version
# Should output: cargo 1.xx.x

rustc --version
# Should output: rustc 1.xx.x
```

### Verify Python Setup

```bash
# Make sure venv is activated
python --version
# Should output: Python 3.8.x or higher

pip list | grep spleeter
# Should show: spleeter x.x.x
```

### Verify FFmpeg

```bash
ffmpeg -version
# Should output FFmpeg version information
```

### Verify System Libraries (Linux)

```bash
pkg-config --modversion alsa
# Should output ALSA version

pkg-config --modversion gtk+-3.0
# Should output GTK3 version
```

### Test Build

```bash
cargo build
# Should complete without errors
```

## Troubleshooting

### Issue: PyO3 can't find Python

**Solution:**

```bash
# Find your Python path
which python3

# Set the environment variable
export PYTHON_SYS_EXECUTABLE=/usr/bin/python3

# Or add to your shell profile (~/.bashrc, ~/.zshrc)
echo 'export PYTHON_SYS_EXECUTABLE=/usr/bin/python3' >> ~/.bashrc
```

### Issue: Spleeter installation fails

**Symptoms:** TensorFlow compatibility issues

**Solution:**

```bash
# Deactivate and remove old venv
deactivate
rm -rf venv

# Create new venv with Python 3.11
python3.11 -m venv venv
source venv/bin/activate

# Install with specific TensorFlow version
pip install tensorflow==2.12.0
pip install spleeter
```

### Issue: FFmpeg not found

**Solution (Ubuntu/Debian):**

```bash
sudo apt-get install ffmpeg
```

**Solution (macOS):**

```bash
brew install ffmpeg
```

**Solution (Windows):**

1. Download FFmpeg from <https://ffmpeg.org/download.html>
2. Extract to C:\ffmpeg
3. Add C:\ffmpeg\bin to PATH environment variable

### Issue: ALSA errors on Linux

**Symptoms:** "ALSA lib pcm.c: Unknown PCM" errors

**Solution:**

```bash
sudo apt-get install libasound2-dev

# Add your user to audio group
sudo usermod -a -G audio $USER

# Log out and log back in
```

### Issue: GTK-related errors on Linux

**Solution:**

```bash
sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### Issue: Slow Spleeter processing

**Recommendation:**

- Spleeter works best with a dedicated GPU (NVIDIA with CUDA)
- For CPU-only processing, expect 2-5x real-time (a 3-minute song takes 6-15 minutes)
- Consider using the 2stems model (faster than 4stems or 5stems)

### Issue: Build fails with linking errors on Windows

**Solution:**

1. Ensure Visual Studio Build Tools are installed
2. Run build from "x64 Native Tools Command Prompt for VS"
3. Or install the full Visual Studio Community Edition

## Next Steps

After successful setup:

1. **Review the project structure** (see README.md)
2. **Start development** - The project structure is ready for implementation
3. **Test Spleeter integration** manually:

   ```bash
   spleeter separate -i test_audio.mp3 -o output
   ```

4. **Run the app** (when implementation begins):

   ```bash
   cargo run
   ```

## Getting Help

If you encounter issues not covered here:

1. Check the [README.md](README.md) troubleshooting section
2. Review [PyO3 documentation](https://pyo3.rs/)
3. Check [Spleeter GitHub issues](https://github.com/deezer/spleeter/issues)
4. Consult team members

## Development Workflow

```bash
# Daily workflow:

# 1. Activate Python environment
source venv/bin/activate

# 2. Pull latest changes
git pull

# 3. Build and run
cargo run

# 4. Before committing, run checks
cargo fmt
cargo clippy
cargo test

# 5. Deactivate when done
deactivate
```
