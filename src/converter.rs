use std::path::{Path, PathBuf};

pub struct ConversionResult {
    pub original: PathBuf,
    pub output: PathBuf,
    pub success: bool,
    pub error: Option<String>,
}

/// Determine the output path for a converted file.
/// If output_folder is empty, places the .jpg next to the original.
/// Otherwise uses output_folder as the destination directory.
fn output_path(original: &Path, output_folder: &str) -> PathBuf {
    let stem = original.file_stem().unwrap_or_default();
    let base_name = format!("{}.jpg", stem.to_string_lossy());
    let dir = if output_folder.is_empty() {
        original.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        PathBuf::from(output_folder)
    };

    // Avoid collisions: if foo.jpg exists, try foo_1.jpg, foo_2.jpg, etc.
    let candidate = dir.join(&base_name);
    if !candidate.exists() {
        return candidate;
    }
    let stem_str = stem.to_string_lossy();
    let mut i = 1u32;
    loop {
        let name = format!("{}_{}.jpg", stem_str, i);
        let candidate = dir.join(&name);
        if !candidate.exists() {
            return candidate;
        }
        i += 1;
    }
}

fn is_heic(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let lower = e.to_lowercase();
            lower == "heic" || lower == "heif"
        })
        .unwrap_or(false)
}

pub fn convert_one(
    source: &Path,
    quality: u8,
    output_folder: &str,
    delete_original: bool,
) -> ConversionResult {
    let out_path = output_path(source, output_folder);

    let result = if is_heic(source) {
        convert_heic(source, &out_path, quality)
    } else {
        convert_standard(source, &out_path, quality)
    };

    match result {
        Ok(()) => {
            if delete_original {
                if let Err(e) = std::fs::remove_file(source) {
                    eprintln!("Warning: converted {} but failed to delete original: {}", source.display(), e);
                }
            }
            ConversionResult {
                original: source.to_path_buf(),
                output: out_path,
                success: true,
                error: None,
            }
        }
        Err(e) => ConversionResult {
            original: source.to_path_buf(),
            output: out_path,
            success: false,
            error: Some(e.to_string()),
        },
    }
}

fn ensure_parent(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn convert_standard(source: &Path, out_path: &Path, quality: u8) -> anyhow::Result<()> {
    use image::codecs::jpeg::JpegEncoder;
    use std::fs::File;
    use std::io::BufWriter;

    ensure_parent(out_path)?;
    let img = image::open(source)?;
    let file = File::create(out_path)?;
    let writer = BufWriter::new(file);
    let encoder = JpegEncoder::new_with_quality(writer, quality);
    img.write_with_encoder(encoder)?;
    Ok(())
}

fn convert_heic(source: &Path, out_path: &Path, quality: u8) -> anyhow::Result<()> {
    ensure_parent(out_path)?;

    // Map quality 0-100 → ffmpeg -q:v 2-31 (2=best, 31=worst)
    let qv = (31.0 - (quality as f32 / 100.0) * 29.0).round() as u32;
    let qv = qv.clamp(2, 31);

    let status = std::process::Command::new(crate::ffmpeg::ffmpeg_path())
        .args([
            "-v", "error",
            "-y",
            "-i", source.to_str().ok_or_else(|| anyhow::anyhow!("non-UTF8 path"))?,
            "-update", "1",
            "-q:v", &qv.to_string(),
            out_path.to_str().ok_or_else(|| anyhow::anyhow!("non-UTF8 output path"))?,
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run ffmpeg: {e}"))?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        let summary: Vec<&str> = stderr.lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        let msg = if summary.is_empty() {
            "ffmpeg exited with non-zero status".to_string()
        } else {
            summary.join("; ")
        };
        anyhow::bail!("{}", msg);
    }

    Ok(())
}

pub async fn convert_batch(
    files: Vec<PathBuf>,
    quality: u8,
    output_folder: String,
    delete_original: bool,
    max_parallel: usize,
) -> Vec<ConversionResult> {
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    let sem = Arc::new(Semaphore::new(max_parallel));
    let mut handles = Vec::with_capacity(files.len());

    for file in files {
        let sem = Arc::clone(&sem);
        let output_folder = output_folder.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let display = file.file_name().unwrap_or_default().to_string_lossy().into_owned();
            println!("Converting: {}", display);
            // image decoding is CPU-bound, run on blocking thread pool
            let result = tokio::task::spawn_blocking(move || {
                convert_one(&file, quality, &output_folder, delete_original)
            })
            .await
            .unwrap_or_else(|e| ConversionResult {
                original: PathBuf::new(),
                output: PathBuf::new(),
                success: false,
                error: Some(format!("task panicked: {}", e)),
            });
            if result.success {
                println!("  -> {}", result.output.file_name().unwrap_or_default().to_string_lossy());
            } else {
                eprintln!("  ERROR: {}", result.error.as_deref().unwrap_or("unknown"));
            }
            result
        }));
    }

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        if let Ok(r) = handle.await {
            results.push(r);
        }
    }
    results
}
