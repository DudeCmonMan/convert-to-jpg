mod config;
mod converter;
mod tui;
mod utils;

use clap::Parser;
use std::path::PathBuf;

const BUILD_VERSION: &str = env!("BUILD_VERSION");

#[derive(Parser, Debug)]
#[command(
    name = "convert-to-jpg",
    version = BUILD_VERSION,
    about = "Convert image files to JPEG"
)]
struct Args {
    /// Files or directories to convert
    #[arg(required = true)]
    paths: Vec<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let (config, config_path) = config::load()?;
    eprintln!("Config: {}", config_path.display());

    // Collect all files from provided paths
    let mut all_files: Vec<PathBuf> = Vec::new();
    for path in &args.paths {
        all_files.extend(utils::collect_files(path));
    }

    // Filter to convertible formats
    all_files.retain(|f| utils::is_convertible(f, &config.formats.extensions));

    if all_files.is_empty() {
        println!("No convertible image files found.");
        return Ok(());
    }

    // Build TUI entries
    let entries: Vec<tui::ImageEntry> = all_files
        .iter()
        .map(|path| {
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown")
                .to_uppercase();
            let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            tui::ImageEntry {
                path: path.clone(),
                filename,
                format,
                file_size,
            }
        })
        .collect();

    // Show TUI — blocking, runs before async conversion work
    let accepted_indices = tui::select_files(&entries)?;

    if accepted_indices.is_empty() {
        println!("Cancelled.");
        return Ok(());
    }

    let selected: Vec<PathBuf> = accepted_indices
        .iter()
        .map(|&i| all_files[i].clone())
        .collect();

    println!("\nConverting {} file(s)...\n", selected.len());

    let results = converter::convert_batch(
        selected,
        config.conversion.quality,
        config.conversion.output_folder.clone(),
        config.conversion.delete_originals,
        config.conversion.max_parallel,
    )
    .await;

    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();

    println!("\nDone. {} converted", succeeded);
    if failed > 0 {
        eprintln!("{} failed:", failed);
        for r in results.iter().filter(|r| !r.success) {
            eprintln!(
                "  {}: {}",
                r.original.display(),
                r.error.as_deref().unwrap_or("unknown error")
            );
        }
    }

    Ok(())
}
