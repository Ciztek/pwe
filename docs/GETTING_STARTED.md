# 🚀 Getting Started Checklist

Use this checklist to ensure your development environment is properly set up.

## ✅ Initial Setup (Do Once)

### 1. System Dependencies

- [ ] Linux: Install build essentials, ALSA, GTK, Python dev headers, FFmpeg

  ```bash
  sudo apt-get install build-essential libasound2-dev libgtk-3-dev python3-dev ffmpeg
  ```

- [ ] macOS: Install Homebrew, Python, pkg-config, FFmpeg

  ```bash
  brew install python@3.11 pkg-config ffmpeg
  ```

- [ ] Windows: Install Visual Studio Build Tools, Python 3.8+, FFmpeg

### 2. Rust Toolchain

- [ ] Install Rust via rustup

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
  ```

- [ ] Verify installation

  ```bash
  rustc --version
  cargo --version
  ```

### 3. Python Environment

- [ ] Create virtual environment

  ```bash
  make setup-python
  # or: python3 -m venv venv
  ```

- [ ] Activate virtual environment

  ```bash
  source venv/bin/activate  # Linux/Mac
  # or: venv\Scripts\activate  # Windows
  ```

- [ ] Install Python dependencies

  ```bash
  make install-python
  # or: pip install -r requirements.txt
  ```

- [ ] Verify Spleeter installation

  ```bash
  spleeter --version
  ```

### 4. Rust Project

- [ ] Build the project

  ```bash
  cargo build
  ```

- [ ] Run the placeholder

  ```bash
  cargo run
  ```

  Should output: "PWE Karaoke - Setup complete!"

### 5. VS Code Setup (Optional but Recommended)

- [ ] Install VS Code
- [ ] Open project in VS Code
- [ ] Install recommended extensions when prompted
  - Or manually: Ctrl+Shift+P → "Extensions: Show Recommended Extensions"
- [ ] Verify Rust Analyzer is working (should see type hints)

## 🔧 Verification Tests

Run these to ensure everything works:

- [ ] **Format check**

  ```bash
  cargo fmt -- --check
  ```

  Should complete without errors

- [ ] **Clippy check**

  ```bash
  cargo clippy
  ```

  Should complete without errors (just the placeholder main.rs)

- [ ] **Build check**

  ```bash
  cargo check
  ```

  Should complete successfully

- [ ] **FFmpeg check**

  ```bash
  ffmpeg -version
  ```

  Should show FFmpeg version

- [ ] **Spleeter check** (with venv activated)

  ```bash
  python -c "import spleeter; print('Spleeter OK')"
  ```

  Should print "Spleeter OK"

## 📚 Documentation Review

Read these before starting development:

- [ ] **README.md** - Project overview and goals
- [ ] **SETUP.md** - Detailed setup instructions
- [ ] **ARCHITECTURE.md** - Technical architecture and design
- [ ] **DEVELOPMENT.md** - Development workflow and best practices
- [ ] **QUICKREF.md** - Quick command reference

## 🎯 Ready to Start?

If all checkboxes are checked, you're ready to start implementing!

### First Development Steps

1. **Activate Python environment** (always do this first)

   ```bash
   source venv/bin/activate
   ```

2. **Create a new branch** for your feature

   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Implement your feature** (refer to ARCHITECTURE.md for structure)

4. **Format and lint** your code

   ```bash
   make all
   ```

5. **Test your changes**

   ```bash
   cargo test
   cargo run
   ```

6. **Commit and push**

   ```bash
   git add .
   git commit -m "feat: your feature description"
   git push
   ```

## 🆘 Troubleshooting

If something doesn't work:

1. Check the **Troubleshooting** section in SETUP.md
2. Review the **Common Issues** section in DEVELOPMENT.md
3. Ensure all system dependencies are installed
4. Verify Python venv is activated
5. Try cleaning and rebuilding:

   ```bash
   cargo clean
   cargo build
   ```

## 📋 Daily Workflow Reminder

Every time you start working:

```bash
# 1. Pull latest changes
git pull

# 2. Activate Python environment
source venv/bin/activate

# 3. Work on your code...

# 4. Before committing
make pre-commit

# 5. Commit and push
git add .
git commit -m "your message"
git push
```

## 🎉 All Set

Your development environment is ready. Happy coding! 🦀🎤
