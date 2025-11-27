# PWE Karaoke - Development Guide

This guide covers code formatting, linting, and development workflow.

## 📝 Code Formatting and Linting

### Rustfmt (Code Formatter)

We use `rustfmt` to automatically format Rust code according to our style guidelines.

**Configuration**: `.rustfmt.toml`

- 100 character line width
- 4 spaces indentation
- Automatic import organization
- Unix line endings

**Usage:**

```bash
# Format all code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Or use make command
make fmt
```

**IDE Integration:**

- VS Code: Formatting happens automatically on save (if you use our workspace settings)
- Manual format: Right-click → Format Document (or Shift+Alt+F)

### Clippy (Linter)

Clippy is Rust's official linter that catches common mistakes and suggests improvements.

**Configuration**:

- `.clippy.toml` for thresholds
- `Cargo.toml` `[lints.clippy]` section for lint levels

**Our Setup:**

- **DENY**: Correctness and suspicious patterns (safety issues)
- **WARN**: Complexity, performance, and style issues
- **ALLOW**: Pedantic rules (too strict for learning)
- **WARN on**: `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`

**Usage:**

```bash
# Run clippy (fails on warnings)
cargo clippy --all-targets --all-features

# Run with suggestions only (doesn't fail)
cargo clippy --all-targets --all-features -- -W clippy::pedantic

# Or use make commands
make lint                  # Strict (fails on warnings)
make lint-suggestions      # Shows suggestions only
```

**Common Clippy Warnings You'll See:**

1. **`unwrap_used`**: Use `?` operator or proper error handling instead of `.unwrap()`

   ```rust
   // ❌ Avoid
   let value = some_option.unwrap();

   // ✅ Better
   let value = some_option?;
   // or
   let value = some_option.expect("meaningful error message");
   ```

2. **`needless_return`**: Rust uses implicit returns

   ```rust
   // ❌ Unnecessary
   fn add(a: i32, b: i32) -> i32 {
       return a + b;
   }

   // ✅ Idiomatic
   fn add(a: i32, b: i32) -> i32 {
       a + b
   }
   ```

3. **`redundant_clone`**: Avoid unnecessary cloning

   ```rust
   // ❌ Unnecessary clone
   let s2 = s1.clone();
   println!("{}", s1);  // s1 not used after

   // ✅ Just move
   let s2 = s1;
   ```

## 🛠️ Development Workflow

### Quick Start

```bash
# 1. Format your code
make fmt

# 2. Check for issues
make lint

# 3. Build the project
make build

# 4. Run the project
make run

# Or do all at once
make all
```

### Using the Makefile

We provide a `Makefile` with convenient shortcuts:

| Command | Description |
|---------|-------------|
| `make help` | Show all available commands |
| `make setup-python` | Create Python virtual environment |
| `make install-python` | Install Spleeter and dependencies |
| `make build` | Build in debug mode |
| `make run` | Build and run |
| `make release` | Build optimized release |
| `make fmt` | Format code |
| `make lint` | Run clippy (strict) |
| `make lint-suggestions` | Run clippy (suggestions only) |
| `make check` | Format check + lint + build |
| `make test` | Run tests |
| `make clean` | Clean build artifacts |
| `make all` | Format + lint + build |
| `make pre-commit` | Format + lint + test |

### Daily Workflow

```bash
# Morning: Pull latest changes
git pull

# Activate Python environment (for Spleeter)
source venv/bin/activate

# Work on your code...

# Before committing:
make pre-commit

# If all passes:
git add .
git commit -m "Your commit message"
git push
```

### VS Code Integration

Our workspace settings (`.vscode/settings.json`) provide:

✅ **Auto-format on save**: Your code is automatically formatted when you save
✅ **Inline errors**: Clippy warnings appear directly in your editor
✅ **Inlay hints**: See type information inline (helpful for learning)
✅ **Auto-completion**: Full Rust and Python IntelliSense
✅ **Python integration**: Automatic venv detection

**Recommended Extensions** (see `.vscode/extensions.json`):

- `rust-analyzer` - Rust language support (REQUIRED)
- `vadimcn.vscode-lldb` - Debugging
- `ms-python.python` - Python support for Spleeter
- `tamasfe.even-better-toml` - TOML syntax
- `usernamehw.errorlens` - Inline error display

### Debugging

**Print Debugging:**

```rust
// Use dbg! macro (removed in release builds)
dbg!(&my_variable);

// Or println with Debug formatting
println!("{:?}", my_variable);

// Or with pretty-print
println!("{:#?}", my_variable);
```

**VS Code Debugger:**

1. Set breakpoints by clicking left of line numbers
2. Press F5 or use Run → Start Debugging
3. Use LLDB extension for full debugging features

**Logging with tracing:**

```rust
use tracing::{info, warn, error, debug};

info!("Application started");
debug!("Debug information: {:?}", data);
warn!("Something might be wrong");
error!("Error occurred: {}", err);
```

## 🔍 Common Issues and Solutions

### Issue: "unwrap() called on None"

**Cause**: Using `.unwrap()` on an `Option` or `Result` that fails

**Solution**: Use proper error handling

```rust
// ❌ Can panic
let value = map.get("key").unwrap();

// ✅ Safe options
let value = map.get("key")?;  // If in function returning Result
let value = map.get("key").ok_or(MyError::KeyNotFound)?;
let value = map.get("key").unwrap_or(&default);
```

### Issue: "borrowed value does not live long enough"

**Cause**: Trying to use a reference that outlives its owner

**Solution**: Review Rust ownership rules

```rust
// ❌ Reference outlives owner
let r;
{
    let x = 5;
    r = &x;
}  // x dropped here
println!("{}", r);  // Error: x no longer exists

// ✅ Extend lifetime or clone
let x = 5;
let r = &x;
println!("{}", r);
```

### Issue: Clippy warnings overwhelming

**Solutions**:

1. Fix one warning at a time
2. Use `make lint-suggestions` instead of `make lint` for softer checking
3. Temporarily allow specific lints:

   ```rust
   #[allow(clippy::too_many_arguments)]
   fn complex_function(a: i32, b: i32, c: i32, ...) { }
   ```

4. Ask team members or consult documentation

### Issue: Formatting conflicts with personal style

Our formatting is team-standard. If something seems wrong:

1. Check if it's a Rust convention (it probably is)
2. Discuss with team if it really needs changing
3. Remember: consistent style > personal preference

## 📚 Learning Resources

### Rust Fundamentals

- [The Rust Book](https://doc.rust-lang.org/book/) - Essential reading
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings) - Interactive exercises

### Clippy & Best Practices

- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/) - All lint explanations
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)

### Project-Specific

- [egui documentation](https://docs.rs/egui/)
- [rodio documentation](https://docs.rs/rodio/)
- [PyO3 guide](https://pyo3.rs/)
- [Tokio tutorial](https://tokio.rs/tokio/tutorial)

## 🎯 Code Review Checklist

Before submitting code for review:

- [ ] Code is formatted (`make fmt`)
- [ ] Clippy passes without warnings (`make lint`)
- [ ] Tests pass (`make test`)
- [ ] No `unwrap()` or `panic!()` in production code (use `?` or proper error handling)
- [ ] Error messages are descriptive
- [ ] Public functions have documentation comments
- [ ] Complex logic has explanatory comments
- [ ] No TODO/FIXME comments unless tracked as issues
- [ ] Commit messages are clear and descriptive

## 🚀 Performance Tips

### When to Optimize

1. First, make it work
2. Then, make it right (clean, maintainable)
3. Finally, make it fast (only if needed)

"Premature optimization is the root of all evil" - Donald Knuth

### Common Optimizations

- Use `&str` instead of `String` when you don't need ownership
- Use `Vec::with_capacity()` if you know the size
- Use iterators instead of collecting into intermediate `Vec`s
- Use `Cow<str>` for string data that might be owned or borrowed
- Profile before optimizing (use `cargo flamegraph`)

## 🤝 Getting Help

### When Stuck

1. Read the compiler error message carefully (Rust errors are very helpful!)
2. Search the error message or clippy warning
3. Check our documentation (README.md, ARCHITECTURE.md, SETUP.md)
4. Ask team members
5. Consult [r/rust](https://reddit.com/r/rust) or [Rust Users Forum](https://users.rust-lang.org/)

### Useful Commands

```bash
# See full error explanation
rustc --explain E0308

# See clippy lint explanation
cargo clippy -- -W clippy::unwrap_used --explain

# Check what clippy would fix
cargo clippy --fix

# Update dependencies
cargo update

# Show dependency tree
cargo tree
```

## 🎓 Tips for Rust Beginners

1. **Fighting the borrow checker is normal** - It gets easier with practice
2. **Read error messages completely** - They often tell you exactly how to fix the issue
3. **Use `cargo check` often** - Faster than full build, catches most errors
4. **Don't fear `clone()`** - It's okay while learning; optimize later
5. **Use `todo!()` and `unimplemented!()` placeholders** - Build incrementally
6. **Write small functions** - Easier to understand ownership and lifetimes
7. **Learn by doing** - Build features, break things, learn from errors

Remember: Everyone struggles with Rust at first. It's worth it! 🦀
