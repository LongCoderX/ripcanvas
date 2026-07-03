use std::path::Path;

use ripcanvas::canvas::{parser::parse_canvas_file, view_model::CanvasViewModel};

#[test]
fn parses_basic_canvas_when_nodes_and_edges_are_present() {
    // Given
    let fixture = Path::new("tests/fixtures/basic.canvas");

    // When
    let canvas = parse_canvas_file(fixture).expect("fixture should parse");

    // Then
    assert_eq!(canvas.nodes.len(), 2);
    assert_eq!(canvas.edges.len(), 1);
    assert_eq!(canvas.nodes[0].id.as_str(), "node-a");
    assert_eq!(canvas.nodes[0].label(), "**Architecture** note");
    assert_eq!(canvas.nodes[0].color.as_deref(), Some("#cdebdc"));
    assert_eq!(canvas.nodes[1].label(), "docs/plan.md");
    assert_eq!(canvas.nodes[1].color.as_deref(), Some("6"));
    assert_eq!(canvas.edges[0].from_side.as_deref(), Some("right"));
    assert_eq!(canvas.edges[0].to_side.as_deref(), Some("left"));
}

#[test]
fn defaults_top_level_collections_when_canvas_is_empty_object() {
    // Given
    let raw = "{}";

    // When
    let canvas =
        ripcanvas::canvas::parser::parse_canvas_str(raw).expect("empty object should parse");

    // Then
    assert!(canvas.nodes.is_empty());
    assert!(canvas.edges.is_empty());
}

#[test]
fn rejects_canvas_when_required_geometry_is_missing() {
    // Given
    let raw = r#"{"nodes":[{"id":"node-a","type":"text","x":0,"y":0,"width":120,"text":"Missing height"}]}"#;

    // When
    let result = ripcanvas::canvas::parser::parse_canvas_str(raw);

    // Then
    assert!(result.is_err());
}

#[test]
fn normalizes_negative_coordinates_with_padding_when_building_view_model() {
    // Given
    let fixture = Path::new("tests/fixtures/negative-coordinates.canvas");
    let canvas = parse_canvas_file(fixture).expect("fixture should parse");

    // When
    let view_model = CanvasViewModel::from_canvas(&canvas);

    // Then
    assert_eq!(view_model.nodes.len(), 2);
    assert_eq!(view_model.nodes[0].x, CanvasViewModel::PADDING);
    assert_eq!(view_model.nodes[0].y, CanvasViewModel::PADDING);
    assert_eq!(view_model.nodes[1].x, 250.0 + CanvasViewModel::PADDING);
    assert_eq!(view_model.nodes[1].y, 120.0 + CanvasViewModel::PADDING);
}

#[test]
fn maps_canvas_colors_and_edge_endpoints_into_view_model() {
    // Given
    let fixture = Path::new("tests/fixtures/basic.canvas");
    let canvas = parse_canvas_file(fixture).expect("fixture should parse");

    // When
    let view_model = CanvasViewModel::from_canvas(&canvas);

    // Then
    assert_eq!(view_model.nodes[0].color, "#cdebdc");
    assert_eq!(view_model.nodes[1].color, "#7852ee");
    assert_eq!(
        view_model.edges[0].from_x,
        view_model.nodes[0].x + view_model.nodes[0].width
    );
    assert_eq!(view_model.edges[0].to_x, view_model.nodes[1].x);
    assert_eq!(view_model.edges[0].from_color, "#cdebdc");
    assert_eq!(view_model.edges[0].to_color, "#7852ee");
}

#[test]
fn maps_node_label_to_card_title_without_duplicating_group_body() {
    // Given
    let raw = r##"{
      "nodes": [
        {
          "id": "node-a",
          "type": "text",
          "x": 0,
          "y": 0,
          "width": 180,
          "height": 120,
          "label": "Decision",
          "text": "Body **markdown**",
          "color": "#ffffff"
        },
        {
          "id": "group-a",
          "type": "group",
          "x": 220,
          "y": 0,
          "width": 240,
          "height": 180,
          "label": "Group Title",
          "color": "#dde6ee"
        }
      ],
      "edges": []
    }"##;

    // When
    let canvas = ripcanvas::canvas::parser::parse_canvas_str(raw).expect("canvas should parse");
    let view_model = CanvasViewModel::from_canvas(&canvas);

    // Then
    assert_eq!(canvas.nodes[0].title(), Some("Decision"));
    assert_eq!(view_model.nodes[0].title, "Decision");
    assert_eq!(view_model.nodes[0].label, "Body **markdown**");
    assert_eq!(view_model.nodes[1].title, "Group Title");
    assert_eq!(view_model.nodes[1].label, "");
}
