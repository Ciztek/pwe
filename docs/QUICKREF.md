# PWE Karaoke - Quick Reference

Quick cheat sheet for common development tasks.

## 🚀 Getting Started

```bash
# First time setup
git clone <repo-url>
cd pwe
make setup-python
source venv/bin/activate
make install-python
cargo build
```

## 📝 Daily Workflow

```bash
# Start working
source venv/bin/activate

# Format, lint, and build
make all

# Or individually
make fmt        # Format code
make lint       # Run lints
make build      # Compile
make run        # Run the app

# Before commit
make pre-commit
```

## 🔧 Common Commands

| Task | Command | Alternative |
|------|---------|-------------|
| Format code | `cargo fmt` | `make fmt` |
| Check format | `cargo fmt -- --check` | `make fmt-check` |
| Lint code | `cargo clippy` | `make lint` |
| Build debug | `cargo build` | `make build` |
| Build release | `cargo build --release` | `make release` |
| Run | `cargo run` | `make run` |
| Test | `cargo test` | `make test` |
| Clean | `cargo clean` | `make clean` |
| Check compilation | `cargo check` | - |
| Update deps | `cargo update` | - |
| View dep tree | `cargo tree` | - |

## 🐍 Python/Spleeter

```bash
# Activate environment
source venv/bin/activate  # Linux/Mac
venv\Scripts\activate     # Windows

# Install/update dependencies
pip install -r requirements.txt

# Test Spleeter
spleeter separate -i audio.mp3 -o output

# Deactivate
deactivate
```

## 🔍 Debugging

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo run

# Run with full backtrace
RUST_BACKTRACE=full cargo run

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## 📊 Clippy Severity Levels

| Level | Meaning | Action |
|-------|---------|--------|
| `deny` | Compilation error | Must fix |
| `warn` | Warning | Should fix |
| `allow` | Ignored | Optional |

## 🛠️ VS Code Shortcuts

| Action | Shortcut |
|--------|----------|
| Format document | `Shift+Alt+F` |
| Go to definition | `F12` |
| Find references | `Shift+F12` |
| Rename symbol | `F2` |
| Show problems | `Ctrl+Shift+M` |
| Command palette | `Ctrl+Shift+P` |
| Quick fix | `Ctrl+.` |

## 🚨 Common Fixes

### Fix all formatting issues

```bash
cargo fmt --all
```

### Fix auto-fixable clippy issues

```bash
cargo clippy --fix
```

### Allow specific warning

```rust
#[allow(clippy::warning_name)]
fn my_function() { }
```

### Explain error code

```bash
rustc --explain E0308
```

## 📦 Dependency Management

```bash
# Add dependency
cargo add <crate-name>

# Add dev dependency
cargo add --dev <crate-name>

# Add with features
cargo add <crate-name> --features feat1,feat2

# Remove dependency
cargo remove <crate-name>

# Update dependencies
cargo update
```

## 🎯 Error Handling Patterns

```rust
// Use ? operator
let value = function_that_returns_result()?;

// Or match
match function_that_returns_result() {
    Ok(value) => { /* use value */ },
    Err(e) => { /* handle error */ },
}

// Unwrap with message
let value = option.expect("meaningful message");

// Provide default
let value = option.unwrap_or(default_value);
let value = option.unwrap_or_else(|| compute_default());
```

## 🔗 Quick Links

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Spleeter Docs](https://github.com/deezer/spleeter)
- [egui Docs](https://docs.rs/egui/)
- [Project README](README.md)
- [Setup Guide](SETUP.md)
- [Architecture](ARCHITECTURE.md)
- [Development Guide](DEVELOPMENT.md)

## 🐛 Troubleshooting

| Problem | Solution |
|---------|----------|
| PyO3 build fails | Set `PYTHON_SYS_EXECUTABLE` env var |
| ALSA errors | Install `libasound2-dev` |
| GTK errors | Install `libgtk-3-dev` |
| Clippy too strict | Use `make lint-suggestions` |
| Formatting wrong | Check `.rustfmt.toml` |

## 📋 Pre-Commit Checklist

- [ ] `cargo fmt` - Code formatted
- [ ] `cargo clippy` - No warnings
- [ ] `cargo test` - Tests pass
- [ ] `cargo build` - Compiles successfully
- [ ] Commit message is descriptive
- [ ] No debug code left (prints, todos)
