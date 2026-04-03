use anyhow::Context;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub conversion: ConversionConfig,
    #[serde(default)]
    pub formats: FormatsConfig,
}

#[derive(Debug, Deserialize)]
pub struct ConversionConfig {
    #[serde(default = "default_quality")]
    pub quality: u8,
    #[serde(default)]
    pub delete_originals: bool,
    #[serde(default)]
    pub output_folder: String,
    #[serde(default = "default_max_parallel")]
    pub max_parallel: usize,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            quality: default_quality(),
            delete_originals: false,
            output_folder: String::new(),
            max_parallel: default_max_parallel(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FormatsConfig {
    #[serde(default = "default_extensions")]
    pub extensions: Vec<String>,
}

impl Default for FormatsConfig {
    fn default() -> Self {
        Self {
            extensions: default_extensions(),
        }
    }
}

fn default_quality() -> u8 {
    90
}
fn default_max_parallel() -> usize {
    4
}
fn default_extensions() -> Vec<String> {
    ["png", "bmp", "tiff", "tif", "webp", "gif", "avif", "heic", "heif"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

pub fn load() -> anyhow::Result<(Config, PathBuf)> {
    let path = crate::utils::find_file_near_binary("config.toml")?;
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok((config, path))
}
