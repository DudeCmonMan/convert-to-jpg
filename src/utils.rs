use anyhow::Context;
use std::path::{Path, PathBuf};

/// Find a named file by checking next to the running binary first, then the current directory.
pub fn find_file_near_binary(filename: &str) -> anyhow::Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(filename);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    let candidate = std::env::current_dir()
        .context("failed to get current directory")?
        .join(filename);
    if candidate.exists() {
        return Ok(candidate);
    }
    anyhow::bail!("{} not found next to binary or in current directory", filename)
}

/// Collect all files from a path. If path is a file, return it directly.
/// If a directory, walk it recursively and return all files.
pub fn collect_files(path: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    collect_files_inner(path, &mut results);
    results
}

fn collect_files_inner(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        out.push(path.to_path_buf());
    } else if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut entries: Vec<_> = entries.flatten().collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                collect_files_inner(&entry.path(), out);
            }
        }
    }
}

/// Check if a file's extension is in the list of convertible formats.
pub fn is_convertible(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let lower = e.to_lowercase();
            extensions.iter().any(|x| x.to_lowercase() == lower)
        })
        .unwrap_or(false)
}

/// Human-readable file size string (e.g. "2.4 MB").
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
