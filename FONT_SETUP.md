# Custom Font Setup (CaskaydiaMono Nerd Font)

## To enable the custom font

### 1. Download the font

Get CaskaydiaMono Nerd Font from:

- <https://www.nerdfonts.com/font-downloads>
- Or: <https://github.com/ryanoasis/nerd-fonts/releases>

### 2. Extract and place the font file

```bash
mkdir -p assets
# Copy the .ttf file (e.g., CaskaydiaCoveNerdFontMono-Regular.ttf)
cp /path/to/CaskaydiaCoveNerdFontMono-Regular.ttf assets/CaskaydiaMono.ttf
```

### 3. Build with custom font feature

```bash
cargo build --features custom-font --release
# Or for Windows:
cargo build --target x86_64-pc-windows-gnu --features custom-font --release
```

## Without custom font

The app will use egui's default fonts which already support:

- UTF-8 characters (accented letters, symbols)
- Basic emoji
- Most international characters

The custom font adds:

- Nerd Font icons (thousands of programming/system icons)
- Better monospace consistency
- Enhanced readability for code-style text
