use crate::canvas::{
    markdown::{MarkdownBlock, obsidian_markdown_blocks, obsidian_markdown_preview},
    model::CanvasDocument,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasViewModel {
    pub nodes: Vec<CanvasNodeView>,
    pub edges: Vec<CanvasEdgeView>,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasNodeView {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub label: String,
    pub editable_label: String,
    pub editable_text: String,
    pub markdown: String,
    pub markdown_blocks: Vec<MarkdownBlockView>,
    pub content_scroll_max: f32,
    pub source: String,
    pub geometry: String,
    pub geometry_x: String,
    pub geometry_y: String,
    pub geometry_w: String,
    pub geometry_h: String,
    pub color: String,
    pub color_raw: String,
    pub text_color: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownBlockView {
    pub kind: String,
    pub text: String,
    pub plain: String,
    pub marker: String,
    pub level: i32,
    pub checked: bool,
    pub indent: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasEdgeView {
    pub from_id: String,
    pub to_id: String,
    pub from_node_x: f32,
    pub from_node_y: f32,
    pub from_node_width: f32,
    pub from_node_height: f32,
    pub to_node_x: f32,
    pub to_node_y: f32,
    pub to_node_width: f32,
    pub to_node_height: f32,
    pub from_x: f32,
    pub from_y: f32,
    pub control_1_x: f32,
    pub control_1_y: f32,
    pub control_2_x: f32,
    pub control_2_y: f32,
    pub to_x: f32,
    pub to_y: f32,
    pub from_color: String,
    pub to_color: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl CanvasViewModel {
    pub const PADDING: f32 = 48.0;

    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            width: 960.0,
            height: 640.0,
        }
    }

    pub fn from_canvas(canvas: &CanvasDocument) -> Self {
        if canvas.nodes.is_empty() {
            return Self::empty();
        }

        let min_x = canvas
            .nodes
            .iter()
            .map(|node| node.x)
            .fold(f32::INFINITY, f32::min);
        let min_y = canvas
            .nodes
            .iter()
            .map(|node| node.y)
            .fold(f32::INFINITY, f32::min);
        let offset_x = Self::PADDING - min_x;
        let offset_y = Self::PADDING - min_y;

        let nodes: Vec<CanvasNodeView> = canvas
            .nodes
            .iter()
            .map(|node| {
                let color_raw = node.color.clone().unwrap_or_default();
                let color = canvas_color_to_hex(node.color.as_deref());
                let title = node.title().unwrap_or("");
                let body = match node.body() {
                    "" if title.is_empty() => node.kind.as_str(),
                    value => value,
                };
                let markdown = obsidian_markdown_preview(body);
                let markdown_blocks: Vec<MarkdownBlockView> = obsidian_markdown_blocks(body)
                    .into_iter()
                    .map(MarkdownBlockView::from)
                    .collect();
                let content_scroll_max = estimated_content_scroll_max(
                    node.height,
                    title,
                    node.title().is_some(),
                    &markdown_blocks,
                );
                CanvasNodeView {
                    id: node.id.clone(),
                    kind: node.kind.clone(),
                    title: title.to_owned(),
                    label: body.to_owned(),
                    editable_label: node.label.clone().unwrap_or_default(),
                    editable_text: node.text.clone().unwrap_or_default(),
                    markdown,
                    markdown_blocks,
                    content_scroll_max,
                    source: node
                        .text
                        .as_deref()
                        .or(node.file.as_deref())
                        .or(node.url.as_deref())
                        .unwrap_or("")
                        .to_owned(),
                    geometry: format!(
                        "x {}  y {}  w {}  h {}",
                        node.x.round(),
                        node.y.round(),
                        node.width.round(),
                        node.height.round()
                    ),
                    geometry_x: node.x.round().to_string(),
                    geometry_y: node.y.round().to_string(),
                    geometry_w: node.width.round().to_string(),
                    geometry_h: node.height.round().to_string(),
                    text_color: readable_text_color(&color).to_owned(),
                    color,
                    color_raw,
                    x: node.x + offset_x,
                    y: node.y + offset_y,
                    width: node.width,
                    height: node.height,
                }
            })
            .collect();

        let edges = canvas
            .edges
            .iter()
            .filter_map(|edge| {
                let from = nodes.iter().find(|node| node.id == edge.from_node)?;
                let to = nodes.iter().find(|node| node.id == edge.to_node)?;
                Some(Self::edge_between(
                    from,
                    to,
                    parse_side(edge.from_side.as_deref()),
                    parse_side(edge.to_side.as_deref()),
                ))
            })
            .collect();

        let width = nodes
            .iter()
            .map(|node| node.x + node.width + Self::PADDING)
            .fold(960.0, f32::max);
        let height = nodes
            .iter()
            .map(|node| node.y + node.height + Self::PADDING)
            .fold(640.0, f32::max);

        Self {
            nodes,
            edges,
            width,
            height,
        }
    }

    fn edge_between(
        from: &CanvasNodeView,
        to: &CanvasNodeView,
        from_side: Option<Side>,
        to_side: Option<Side>,
    ) -> CanvasEdgeView {
        let from_center = center(from);
        let to_center = center(to);
        let from_side = from_side.unwrap_or_else(|| side_toward(from_center, to_center));
        let to_side = to_side.unwrap_or_else(|| side_toward(to_center, from_center));
        let (from_x, from_y) = midpoint_on_side(from, from_side);
        let (to_x, to_y) = midpoint_on_side(to, to_side);
        let (from_dx, from_dy) = side_normal(from_side);
        let (to_dx, to_dy) = side_normal(to_side);
        let distance = ((to_x - from_x).hypot(to_y - from_y)).clamp(80.0, 260.0);
        let pull = distance * 0.42;

        CanvasEdgeView {
            from_id: from.id.clone(),
            to_id: to.id.clone(),
            from_node_x: from.x,
            from_node_y: from.y,
            from_node_width: from.width,
            from_node_height: from.height,
            to_node_x: to.x,
            to_node_y: to.y,
            to_node_width: to.width,
            to_node_height: to.height,
            from_x,
            from_y,
            control_1_x: from_x + from_dx * pull,
            control_1_y: from_y + from_dy * pull,
            control_2_x: to_x + to_dx * pull,
            control_2_y: to_y + to_dy * pull,
            to_x,
            to_y,
            from_color: from.color.clone(),
            to_color: to.color.clone(),
        }
    }
}

fn estimated_content_scroll_max(
    node_height: f32,
    _title: &str,
    has_title: bool,
    blocks: &[MarkdownBlockView],
) -> f32 {
    let header_height = if has_title { 34.0 } else { 0.0 };
    let visible_height = (node_height - header_height - 26.0).max(48.0);
    let content_height: f32 = blocks
        .iter()
        .map(|block| match block.kind.as_str() {
            "heading" => (30.0 - block.level as f32 * 2.0).max(20.0),
            "code" => 56.0,
            "table" => 52.0,
            "callout" => 42.0,
            "embed" => 38.0,
            "rule" => 12.0,
            _ => 28.0,
        })
        .sum::<f32>()
        + blocks.len().saturating_sub(1) as f32 * 5.0;

    (content_height - visible_height).max(0.0)
}

impl From<MarkdownBlock> for MarkdownBlockView {
    fn from(block: MarkdownBlock) -> Self {
        Self {
            kind: block.kind.as_str().to_owned(),
            text: block.text,
            plain: block.plain,
            marker: block.marker,
            level: i32::from(block.level),
            checked: block.checked,
            indent: i32::from(block.indent),
        }
    }
}

fn center(node: &CanvasNodeView) -> (f32, f32) {
    (node.x + node.width / 2.0, node.y + node.height / 2.0)
}

fn parse_side(side: Option<&str>) -> Option<Side> {
    match side {
        Some("top") => Some(Side::Top),
        Some("right") => Some(Side::Right),
        Some("bottom") => Some(Side::Bottom),
        Some("left") => Some(Side::Left),
        _ => None,
    }
}

fn side_toward(from: (f32, f32), to: (f32, f32)) -> Side {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    if dx.abs() >= dy.abs() {
        if dx >= 0.0 { Side::Right } else { Side::Left }
    } else if dy >= 0.0 {
        Side::Bottom
    } else {
        Side::Top
    }
}

fn midpoint_on_side(node: &CanvasNodeView, side: Side) -> (f32, f32) {
    let min_x = node.x;
    let max_x = node.x + node.width;
    let min_y = node.y;
    let max_y = node.y + node.height;
    let center_x = node.x + node.width / 2.0;
    let center_y = node.y + node.height / 2.0;
    match side {
        Side::Top => (center_x.clamp(min_x, max_x), min_y),
        Side::Right => (max_x, center_y.clamp(min_y, max_y)),
        Side::Bottom => (center_x.clamp(min_x, max_x), max_y),
        Side::Left => (min_x, center_y.clamp(min_y, max_y)),
    }
}

fn side_normal(side: Side) -> (f32, f32) {
    match side {
        Side::Top => (0.0, -1.0),
        Side::Right => (1.0, 0.0),
        Side::Bottom => (0.0, 1.0),
        Side::Left => (-1.0, 0.0),
    }
}

fn canvas_color_to_hex(color: Option<&str>) -> String {
    match color.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) if value.starts_with('#') => normalize_hex(value),
        Some("1") => "#e93147".to_owned(),
        Some("2") => "#ec7500".to_owned(),
        Some("3") => "#e0ac00".to_owned(),
        Some("4") => "#08b94e".to_owned(),
        Some("5") => "#00bfbc".to_owned(),
        Some("6") => "#7852ee".to_owned(),
        Some(value) => normalize_hex(value),
        None => "#ffffff".to_owned(),
    }
}

fn normalize_hex(value: &str) -> String {
    let trimmed = value.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if hex.len() == 3 && hex.chars().all(|char| char.is_ascii_hexdigit()) {
        let mut expanded = String::from("#");
        for char in hex.chars() {
            expanded.push(char);
            expanded.push(char);
        }
        expanded.to_lowercase()
    } else if hex.len() == 6 && hex.chars().all(|char| char.is_ascii_hexdigit()) {
        format!("#{hex}").to_lowercase()
    } else {
        "#ffffff".to_owned()
    }
}

fn readable_text_color(background: &str) -> &'static str {
    let Some((r, g, b)) = parse_hex_rgb(background) else {
        return "#16201b";
    };
    let luminance = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    if luminance < 142.0 {
        "#ffffff"
    } else {
        "#16201b"
    }
}

fn parse_hex_rgb(value: &str) -> Option<(u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_connects_to_node_boundaries() {
        let from = CanvasNodeView {
            id: "a".to_owned(),
            kind: "text".to_owned(),
            title: String::new(),
            label: "A".to_owned(),
            editable_label: String::new(),
            editable_text: "A".to_owned(),
            markdown: "A".to_owned(),
            markdown_blocks: Vec::new(),
            content_scroll_max: 0.0,
            source: "A".to_owned(),
            geometry: String::new(),
            geometry_x: "10".to_owned(),
            geometry_y: "10".to_owned(),
            geometry_w: "100".to_owned(),
            geometry_h: "80".to_owned(),
            color: "#ffffff".to_owned(),
            color_raw: String::new(),
            text_color: "#16201b".to_owned(),
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 80.0,
        };
        let to = CanvasNodeView {
            id: "b".to_owned(),
            x: 240.0,
            ..from.clone()
        };

        let edge = CanvasViewModel::edge_between(&from, &to, None, None);

        assert_eq!(edge.from_x, 110.0);
        assert_eq!(edge.to_x, 240.0);
        assert_eq!(edge.from_y, 50.0);
        assert_eq!(edge.to_y, 50.0);
        assert_eq!(edge.from_id, "a");
        assert_eq!(edge.to_id, "b");
    }
}
