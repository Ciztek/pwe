# Icon Placeholder

The installer needs an icon at `assets/icon.png`.

## Icon Requirements

- **Format**: PNG
- **Size**: 512x512 pixels (will be scaled automatically)
- **Transparency**: Supported
- **Color**: RGB or RGBA

## Platform-Specific Icons

cargo-bundle will automatically generate:

- **Windows**: .ico file (16x16, 32x32, 48x48, 256x256)
- **macOS**: .icns file (multiple sizes)
- **Linux**: PNG files (various sizes)

## Quick Icon Creation

If you don't have an icon yet, you can create a placeholder:

```bash
# Using ImageMagick
convert -size 512x512 xc:transparent -font Arial -pointsize 200 \
        -fill '#A82028' -gravity center -annotate +0+0 'P' \
        assets/icon.png

# Or use any image editor to create a 512x512 PNG
```

## Replace This File

Once you have your icon, replace `assets/icon.png` and rebuild the installers.
