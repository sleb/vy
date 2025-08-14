use std::{fs, path::Path};

use anyhow::{Context, Result};
use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Prefs {
    pub llm_api_key: String,
    #[serde(default)]
    pub google_api_key: String,
    #[serde(default)]
    pub google_search_engine_id: String,
}

pub fn load_prefs(path: &Path) -> Result<Prefs> {
    let prefs = Config::builder()
        .add_source(config::File::from(path))
        .add_source(config::Environment::with_prefix("VY"))
        .build()?
        .try_deserialize::<Prefs>()
        .with_context(|| {
            format!(
                "Failed to deserialize prefs from config file at {}",
                path.display()
            )
        })?;

    Ok(prefs)
}

pub fn save_prefs(prefs: &Prefs, path: &Path) -> Result<()> {
    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir).with_context(|| {
            format!(
                "Failed to create parent directories for prefs file at {}",
                path.display()
            )
        })?;
    }

    let toml_string = toml::to_string_pretty(prefs).context("Failed to serialize prefs")?;

    fs::write(path, toml_string)
        .with_context(|| format!("Failed to write prefs to file at {}", path.display()))?;

    Ok(())
}
