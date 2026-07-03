use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::canvas::{model::CanvasDocument, parser::parse_canvas_file};

#[derive(Debug, Parser)]
#[command(
    name = "rocv",
    version,
    about = "RipCanvas",
    long_about = "RipCanvas - A fast Rust viewer for Obsidian Canvas"
)]
pub struct Cli {
    /// Optional Obsidian .canvas file to open.
    pub canvas_path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum LaunchDocument {
    Empty,
    Loaded {
        path: PathBuf,
        canvas: CanvasDocument,
    },
}

impl Cli {
    /// Resolve CLI arguments into the document the viewer should launch with.
    ///
    /// # Errors
    /// Returns an error when the provided path does not exist, is not a file, is
    /// not a `.canvas` file, or cannot be parsed as Obsidian Canvas JSON.
    pub fn resolve_launch(&self) -> Result<LaunchDocument> {
        let Some(path) = &self.canvas_path else {
            return Ok(LaunchDocument::Empty);
        };

        if !path.exists() {
            bail!("Canvas file not found: {}", path.display());
        }
        if !path.is_file() {
            bail!("Canvas path is not a file: {}", path.display());
        }
        match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("canvas") => {}
            Some(_) | None => bail!("Expected a .canvas file: {}", path.display()),
        }

        let canvas = parse_canvas_file(path)
            .with_context(|| format!("Failed to load canvas file: {}", path.display()))?;
        Ok(LaunchDocument::Loaded {
            path: path.clone(),
            canvas,
        })
    }
}
