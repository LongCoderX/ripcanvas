use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::canvas::{export::export_canvas_png, model::CanvasDocument, parser::parse_canvas_file};

#[derive(Debug, Parser)]
#[command(
    name = "rocv",
    version,
    about = "RipCanvas",
    long_about = "RipCanvas - A fast Rust viewer for Obsidian Canvas"
)]
pub struct Cli {
    /// Export the canvas to a PNG image instead of opening the viewer.
    #[arg(long, value_name = "PNG_PATH")]
    pub export: Option<PathBuf>,

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

#[derive(Debug)]
pub enum CliAction {
    Launch(LaunchDocument),
    Export { input: PathBuf, output: PathBuf },
}

impl Cli {
    pub fn run(self) -> Result<CliAction> {
        if let Some(output) = &self.export {
            let Some(input) = &self.canvas_path else {
                bail!("Export requires a .canvas file path");
            };
            let input = validate_canvas_path(input)?;
            let canvas = parse_canvas_file(&input)
                .with_context(|| format!("Failed to load canvas file: {}", input.display()))?;
            export_canvas_png(&canvas, output)
                .with_context(|| format!("Failed to export image: {}", output.display()))?;
            return Ok(CliAction::Export {
                input,
                output: output.clone(),
            });
        }

        Ok(CliAction::Launch(self.resolve_launch()?))
    }

    /// Resolve CLI arguments into the document the viewer should launch with.
    ///
    /// # Errors
    /// Returns an error when the provided path does not exist, is not a file, is
    /// not a `.canvas` file, or cannot be parsed as Obsidian Canvas JSON.
    pub fn resolve_launch(&self) -> Result<LaunchDocument> {
        let Some(path) = &self.canvas_path else {
            return Ok(LaunchDocument::Empty);
        };

        let path = validate_canvas_path(path)?;
        let canvas = parse_canvas_file(&path)
            .with_context(|| format!("Failed to load canvas file: {}", path.display()))?;
        Ok(LaunchDocument::Loaded {
            path: path.to_path_buf(),
            canvas,
        })
    }
}

fn validate_canvas_path(path: &std::path::Path) -> Result<PathBuf> {
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

    Ok(path.to_path_buf())
}
