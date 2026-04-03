# convert-to-jpg

A fast, self-contained command-line tool that converts image files to JPEG. It supports a wide range of formats including PNG, BMP, TIFF, WebP, GIF, AVIF, HEIC, and HEIF. HEIC/HEIF conversion is handled by an embedded FFmpeg binary, so no external dependencies are needed.

I wanted a very simple library with no external dependencies and no extra fluff. It literally just does as advertised (with no advertisements, heh).

<img width="774" height="458" alt="image" src="https://github.com/user-attachments/assets/4e3b6b1f-dbc7-4566-ba4b-fa282b558e8a" />

## Supported Formats

PNG, BMP, TIFF, TIF, WebP, GIF, AVIF, HEIC, HEIF

## Usage

```
convert-to-jpg <files or directories>...
```

Pass one or more files or directories. Directories are scanned recursively for convertible images. An interactive TUI lets you review and select which files to convert before processing begins.

### Examples

```bash
# Convert a single file
convert-to-jpg photo.png

# Convert multiple files
convert-to-jpg image1.heic image2.webp image3.avif

# Convert all images in a directory
convert-to-jpg ./photos/

# Mix files and directories
convert-to-jpg photo.png ./vacation-photos/ screenshot.bmp
```

## Configuration

Place a `config.toml` next to the binary to customize behavior:

```toml
[conversion]
quality = 90              # JPEG quality (0-100)
delete_originals = false  # Delete source files after conversion
output_folder = ""        # Output directory ("" = same as source)
max_parallel = 4          # Max concurrent conversions

[formats]
extensions = ["png", "bmp", "tiff", "tif", "webp", "gif", "avif", "heic", "heif"]
```

All settings have sensible defaults. The config file is optional.

## Windows Context Menu (Send To)

You can add convert-to-jpg to the Windows right-click "Send to" menu for quick access:

1. Create a file called `Convert to JPG.bat` next to `convert-to-jpg.exe` with this content:

   ```bat
   @echo off
   "%~dp0convert-to-jpg.exe" %*
   pause
   ```

2. Press `Win+R`, type `shell:sendto`, and press Enter.

3. Create a shortcut to `Convert to JPG.bat` in the Send To folder.

Now you can select files or folders in Explorer, right-click, and choose **Send to > Convert to JPG**.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

This software bundles FFmpeg binaries. See [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) for attribution and licensing details.
