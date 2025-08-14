use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use log::debug;
use rig::{client::completion::CompletionClientDyn, providers::openai};
use vy::Vy;

use crate::prefs::{self, Prefs};

static DEFAULT_PREFS_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

fn default_prefs_path() -> Option<&'static Path> {
    DEFAULT_PREFS_PATH
        .get_or_init(|| {
            directories::ProjectDirs::from("vy", "", "")
                .map(|proj_dirs| proj_dirs.config_dir().join("prefs.toml"))
        })
        .as_deref()
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
    /// Optional prefs path
    #[clap(long, value_parser)]
    prefs_path: Option<PathBuf>,
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        let prefs = self.load_prefs()?;

        let client = openai::Client::builder(&prefs.llm_api_key)
            .build()
            .context("Failed to create LLM client")?;

        let agent = client.agent("gpt-5-nano").preamble("you are a helpful, female, chatbot with a slightly snarky tone. You provide thoughtful and helpful resposes to the user's queries").build();

        let vy = Vy::new(agent);
        vy.chat().await.context("Failed to start Vy chatbot")?;

        Ok(())
    }

    fn load_prefs(&self) -> Result<Prefs> {
        let prefs_path = self
            .prefs_path
            .as_deref()
            .or(default_prefs_path())
            .context("Please specify a prefs path via --prefs-path")?;
        debug!("prefs_path: {prefs_path:?}");

        let prefs = prefs::load_prefs(prefs_path)
            .with_context(|| format!("Failed to load prefs from path: {}", prefs_path.display()))?;
        debug!("prefs: {prefs:?}");

        Ok(prefs)
    }
}
