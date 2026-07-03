use std::{fs, path::Path};

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;

/// Update a canvas node's editable Obsidian content fields in-place.
///
/// `label` is removed when empty, matching Obsidian's optional field shape.
/// `text` is written as a string so a text node can intentionally be cleared.
///
/// # Errors
/// Returns an error when the canvas JSON cannot be read, parsed, updated, or written.
pub fn update_node_content(path: &Path, node_id: &str, label: &str, text: &str) -> Result<()> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read canvas file: {}", path.display()))?;
    let mut document: Value = serde_json::from_str(&raw).context("Invalid Obsidian Canvas JSON")?;
    update_node_content_value(&mut document, node_id, label, text)?;
    let formatted =
        serde_json::to_string_pretty(&document).context("Failed to format canvas JSON")?;
    fs::write(path, format!("{formatted}\n"))
        .with_context(|| format!("Failed to write canvas file: {}", path.display()))
}

pub(crate) fn update_node_content_value(
    document: &mut Value,
    node_id: &str,
    label: &str,
    text: &str,
) -> Result<()> {
    let Some(nodes) = document.get_mut("nodes").and_then(Value::as_array_mut) else {
        bail!("Canvas JSON does not contain a nodes array");
    };

    let Some(node) = nodes
        .iter_mut()
        .find(|node| node.get("id").and_then(Value::as_str) == Some(node_id))
    else {
        bail!("Canvas node not found: {node_id}");
    };
    let Some(node_object) = node.as_object_mut() else {
        return Err(anyhow!("Canvas node is not an object: {node_id}"));
    };

    let label = label.trim();
    if label.is_empty() {
        node_object.remove("label");
    } else {
        node_object.insert("label".to_owned(), Value::String(label.to_owned()));
    }
    node_object.insert("text".to_owned(), Value::String(text.to_owned()));

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn updates_text_and_label_for_matching_node() {
        let mut document = json!({
            "nodes": [
                { "id": "a", "type": "text", "text": "Old", "label": "Old label" },
                { "id": "b", "type": "text", "text": "Keep" }
            ],
            "edges": []
        });

        update_node_content_value(&mut document, "a", "New label", "New **text**")
            .expect("node should update");

        assert_eq!(document["nodes"][0]["label"], "New label");
        assert_eq!(document["nodes"][0]["text"], "New **text**");
        assert_eq!(document["nodes"][1]["text"], "Keep");
    }

    #[test]
    fn removes_empty_label_but_keeps_empty_text() {
        let mut document = json!({
            "nodes": [
                { "id": "a", "type": "text", "text": "Old", "label": "Old label" }
            ],
            "edges": []
        });

        update_node_content_value(&mut document, "a", "   ", "").expect("node should update");

        assert!(document["nodes"][0].get("label").is_none());
        assert_eq!(document["nodes"][0]["text"], "");
    }
}
