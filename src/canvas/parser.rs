use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::canvas::model::CanvasDocument;

/// Parse an Obsidian Canvas document from UTF-8 JSON text.
///
/// # Errors
/// Returns an error when the text is not valid Obsidian Canvas JSON.
pub fn parse_canvas_str(raw: &str) -> Result<CanvasDocument> {
    serde_json::from_str(raw).context("Invalid Obsidian Canvas JSON")
}

/// Parse an Obsidian Canvas document from disk.
///
/// # Errors
/// Returns an error when the file cannot be read or parsed.
pub fn parse_canvas_file(path: &Path) -> Result<CanvasDocument> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read canvas file: {}", path.display()))?;
    parse_canvas_str(&raw)
}
