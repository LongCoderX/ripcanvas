use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanvasDocument {
    #[serde(default)]
    pub nodes: Vec<CanvasNode>,
    #[serde(default)]
    pub edges: Vec<CanvasEdge>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanvasNode {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

impl CanvasNode {
    pub fn label(&self) -> &str {
        self.label
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .or(self.text.as_deref())
            .or(self.file.as_deref())
            .or(self.url.as_deref())
            .unwrap_or(self.kind.as_str())
    }

    pub fn title(&self) -> Option<&str> {
        self.label
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }

    pub fn body(&self) -> &str {
        self.text
            .as_deref()
            .or(self.file.as_deref())
            .or(self.url.as_deref())
            .unwrap_or("")
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanvasEdge {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    #[serde(default)]
    pub from_side: Option<String>,
    #[serde(default)]
    pub to_side: Option<String>,
}
