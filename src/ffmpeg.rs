use std::path::PathBuf;
use std::sync::OnceLock;

#[cfg(not(windows))]
static EMBEDDED_FFMPEG: &[u8] = include_bytes!("../assets/linux/ffmpeg.xz");

#[cfg(windows)]
static EMBEDDED_FFMPEG: &[u8] = include_bytes!("../assets/win/ffmpeg.exe.xz");

/// Extract and decompress the embedded ffmpeg to a temp file on first call, reuse after.
pub fn ffmpeg_path() -> &'static PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        #[cfg(not(windows))]
        let path = std::env::temp_dir().join("convert-to-jpg-ffmpeg");
        #[cfg(windows)]
        let path = std::env::temp_dir().join("convert-to-jpg-ffmpeg.exe");

        if !path.exists() {
            eprintln!("Extracting ffmpeg to {}...", path.display());
            let mut decompressed = Vec::new();
            lzma_rs::xz_decompress(
                &mut std::io::Cursor::new(EMBEDDED_FFMPEG),
                &mut decompressed,
            )
            .expect("failed to decompress embedded ffmpeg");

            std::fs::write(&path, &decompressed)
                .expect("failed to extract embedded ffmpeg to temp directory");

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                    .expect("failed to set ffmpeg executable permissions");
            }
        }

        path
    })
}
