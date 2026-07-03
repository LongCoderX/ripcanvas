use std::{
    fs,
    path::{Path, PathBuf},
};

use ab_glyph::{Font, FontArc, ScaleFont, point};
use anyhow::{Context, Result};
use font8x8::UnicodeFonts;
use fontdb::{Database, Family, Query, Source};
use image::{ImageBuffer, Rgba, RgbaImage};

use crate::canvas::{model::CanvasDocument, view_model::CanvasViewModel};

const BACKGROUND: Rgba<u8> = Rgba([234, 241, 237, 255]);
const CANVAS_BACKGROUND: Rgba<u8> = Rgba([249, 251, 250, 255]);
const BORDER: Rgba<u8> = Rgba([188, 205, 197, 255]);
const TEXT: Rgba<u8> = Rgba([24, 35, 31, 255]);
const EDGE: Rgba<u8> = Rgba([73, 109, 93, 210]);
const MAX_EXPORT_SIDE: f32 = 4096.0;

pub fn export_canvas_png(canvas: &CanvasDocument, path: &Path) -> Result<()> {
    let view_model = CanvasViewModel::from_canvas(canvas);
    export_view_model_png(&view_model, path)
}

pub fn export_view_model_png(view_model: &CanvasViewModel, path: &Path) -> Result<()> {
    let scale = export_scale(view_model.width, view_model.height);
    let width = (view_model.width * scale).ceil().max(1.0) as u32;
    let height = (view_model.height * scale).ceil().max(1.0) as u32;
    let mut image = ImageBuffer::from_pixel(width, height, BACKGROUND);
    fill_rect(
        &mut image,
        0.0,
        0.0,
        view_model.width * scale,
        view_model.height * scale,
        CANVAS_BACKGROUND,
    );

    for edge in &view_model.edges {
        draw_cubic(
            &mut image,
            edge.from_x * scale,
            edge.from_y * scale,
            edge.control_1_x * scale,
            edge.control_1_y * scale,
            edge.control_2_x * scale,
            edge.control_2_y * scale,
            edge.to_x * scale,
            edge.to_y * scale,
            EDGE,
        );
    }

    let font = load_export_font();
    for node in &view_model.nodes {
        let x = node.x * scale;
        let y = node.y * scale;
        let w = node.width * scale;
        let h = node.height * scale;
        let color = parse_hex_color(&node.color).unwrap_or(Rgba([255, 255, 255, 255]));
        let text_color = parse_hex_color(&node.text_color).unwrap_or(TEXT);
        fill_rect(&mut image, x, y, w, h, color);
        stroke_rect(&mut image, x, y, w, h, BORDER);

        if node.title.is_empty() {
            fill_rect(
                &mut image,
                x,
                y,
                w,
                (7.0 * scale).max(4.0),
                color_with_alpha(color, 230),
            );
        } else {
            let title_h = (30.0 * scale).max(20.0);
            fill_rect(
                &mut image,
                x,
                y,
                w,
                title_h.min(h),
                Rgba([255, 255, 255, 92]),
            );
            draw_text(
                &mut image,
                font.as_ref(),
                &node.title,
                x + 12.0 * scale,
                y + 20.0 * scale,
                (13.0 * scale).clamp(10.0, 20.0),
                w - 24.0 * scale,
                1,
                text_color,
            );
        }

        let body = clean_markdown(&node.label);
        if !body.is_empty() {
            let title_offset = if node.title.is_empty() { 0.0 } else { 32.0 };
            draw_text(
                &mut image,
                font.as_ref(),
                &body,
                x + 12.0 * scale,
                y + (22.0 + title_offset) * scale,
                (14.0 * scale).clamp(10.0, 18.0),
                w - 24.0 * scale,
                ((h - (30.0 + title_offset) * scale) / ((18.0 * scale).max(12.0)))
                    .floor()
                    .max(1.0) as usize,
                text_color,
            );
        }
    }

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create export directory: {}", parent.display()))?;
    }
    image
        .save(path)
        .with_context(|| format!("Failed to write PNG: {}", path.display()))
}

fn export_scale(width: f32, height: f32) -> f32 {
    let largest = width.max(height).max(1.0);
    (MAX_EXPORT_SIDE / largest).min(1.5).max(0.35)
}

fn load_export_font() -> Option<FontArc> {
    for path in common_font_paths() {
        if let Ok(data) = fs::read(path) {
            if let Ok(font) = FontArc::try_from_vec(data) {
                return Some(font);
            }
        }
    }

    let mut database = Database::new();
    database.load_system_fonts();
    let families = [
        Family::Name("Microsoft YaHei UI"),
        Family::Name("Microsoft YaHei"),
        Family::Name("Segoe UI"),
        Family::Name("Noto Sans CJK SC"),
        Family::Name("Noto Sans SC"),
        Family::SansSerif,
    ];
    for family in families {
        let query = Query {
            families: &[family],
            ..Query::default()
        };
        let Some(id) = database.query(&query) else {
            continue;
        };
        let Some((source, _index)) = database.face_source(id) else {
            continue;
        };
        if let Some(font) = font_from_source(source) {
            return Some(font);
        }
    }
    for face in database.faces() {
        let Some((source, _index)) = database.face_source(face.id) else {
            continue;
        };
        if let Some(font) = font_from_source(source) {
            return Some(font);
        }
    }
    None
}

fn common_font_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(windir) = std::env::var_os("WINDIR") {
        let fonts = PathBuf::from(windir).join("Fonts");
        paths.push(fonts.join("segoeui.ttf"));
        paths.push(fonts.join("arial.ttf"));
        paths.push(fonts.join("tahoma.ttf"));
        paths.push(fonts.join("msyh.ttc"));
    }
    paths.extend([
        PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
        PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
        PathBuf::from("/System/Library/Fonts/Supplemental/Arial.ttf"),
    ]);
    paths
}

fn font_from_source(source: Source) -> Option<FontArc> {
    let data = match source {
        Source::Binary(data) => data.as_ref().as_ref().to_vec(),
        Source::File(path) => fs::read(path).ok()?,
        Source::SharedFile(_, data) => data.as_ref().as_ref().to_vec(),
    };
    FontArc::try_from_vec(data).ok()
}

fn draw_text(
    image: &mut RgbaImage,
    font: Option<&FontArc>,
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    max_width: f32,
    max_lines: usize,
    color: Rgba<u8>,
) {
    let Some(font) = font else {
        draw_bitmap_text(
            image,
            text,
            x,
            y - size * 0.75,
            size,
            max_width,
            max_lines,
            color,
        );
        return;
    };
    let lines = wrap_text(font, text, size, max_width, max_lines);
    let scaled = font.as_scaled(size);
    let line_height = (scaled.ascent() - scaled.descent() + scaled.line_gap()).max(size + 3.0);
    for (line_index, line) in lines.iter().enumerate() {
        let mut pen_x = x;
        let baseline = y + line_index as f32 * line_height;
        let mut previous = None;
        for character in line.chars() {
            let glyph_id = scaled.glyph_id(character);
            if let Some(previous_id) = previous {
                pen_x += scaled.kern(previous_id, glyph_id);
            }
            let glyph = glyph_id.with_scale_and_position(size, point(pen_x, baseline));
            if let Some(outlined) = font.outline_glyph(glyph) {
                outlined.draw(|glyph_x, glyph_y, coverage| {
                    blend_pixel(image, glyph_x as i32, glyph_y as i32, color, coverage);
                });
            }
            pen_x += scaled.h_advance(glyph_id);
            previous = Some(glyph_id);
        }
    }
    draw_bitmap_text(
        image,
        text,
        x,
        y - size * 0.75,
        size,
        max_width,
        max_lines,
        color,
    );
}

fn draw_bitmap_text(
    image: &mut RgbaImage,
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    max_width: f32,
    max_lines: usize,
    color: Rgba<u8>,
) {
    let pixel = (size / 8.0).max(1.0).round() as i32;
    let char_width = pixel * 8;
    let line_height = pixel * 10;
    let max_chars = ((max_width / char_width as f32).floor() as usize).max(1);
    let mut lines = Vec::new();
    for raw_line in text.lines() {
        let mut line = String::new();
        for ch in raw_line.chars() {
            if line.chars().count() >= max_chars {
                lines.push(line);
                line = String::new();
                if lines.len() >= max_lines {
                    break;
                }
            }
            line.push(if ch.is_ascii() { ch } else { '?' });
        }
        if !line.is_empty() && lines.len() < max_lines {
            lines.push(line);
        }
        if lines.len() >= max_lines {
            break;
        }
    }

    for (line_index, line) in lines.iter().enumerate() {
        for (char_index, character) in line.chars().enumerate() {
            let glyph = font8x8::BASIC_FONTS
                .get(character)
                .or_else(|| font8x8::BASIC_FONTS.get('?'));
            let Some(glyph) = glyph else {
                continue;
            };
            let base_x = x as i32 + char_index as i32 * char_width;
            let base_y = y as i32 + line_index as i32 * line_height;
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if bits & (1 << col) != 0 {
                        fill_rect(
                            image,
                            (base_x + col * pixel) as f32,
                            (base_y + row as i32 * pixel) as f32,
                            pixel as f32,
                            pixel as f32,
                            color,
                        );
                    }
                }
            }
        }
    }
}

fn wrap_text(
    font: &FontArc,
    text: &str,
    size: f32,
    max_width: f32,
    max_lines: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in text.lines() {
        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if text_width(font, &candidate, size) <= max_width || current.is_empty() {
                current = candidate;
            } else {
                lines.push(current);
                current = word.to_owned();
                if lines.len() >= max_lines {
                    return lines;
                }
            }
        }
        if !current.is_empty() {
            lines.push(current);
            if lines.len() >= max_lines {
                return lines;
            }
        }
    }
    lines
}

fn text_width(font: &FontArc, text: &str, size: f32) -> f32 {
    let scaled = font.as_scaled(size);
    let mut width = 0.0;
    let mut previous = None;
    for character in text.chars() {
        let glyph_id = scaled.glyph_id(character);
        if let Some(previous_id) = previous {
            width += scaled.kern(previous_id, glyph_id);
        }
        width += scaled.h_advance(glyph_id);
        previous = Some(glyph_id);
    }
    width
}

fn clean_markdown(text: &str) -> String {
    text.replace("**", "")
        .replace("__", "")
        .replace('`', "")
        .replace('[', "")
        .replace(']', "")
}

fn draw_cubic(
    image: &mut RgbaImage,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    color: Rgba<u8>,
) {
    let mut previous = (x0, y0);
    for step in 1..=56 {
        let t = step as f32 / 56.0;
        let point = cubic_point(t, x0, y0, x1, y1, x2, y2, x3, y3);
        draw_thick_line(image, previous.0, previous.1, point.0, point.1, 3.0, color);
        previous = point;
    }
    fill_circle(image, x3, y3, 5.0, color);
}

fn cubic_point(
    t: f32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
) -> (f32, f32) {
    let mt = 1.0 - t;
    let a = mt * mt * mt;
    let b = 3.0 * mt * mt * t;
    let c = 3.0 * mt * t * t;
    let d = t * t * t;
    (
        a * x0 + b * x1 + c * x2 + d * x3,
        a * y0 + b * y1 + c * y2 + d * y3,
    )
}

fn draw_thick_line(
    image: &mut RgbaImage,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    radius: f32,
    color: Rgba<u8>,
) {
    let distance = (x1 - x0).hypot(y1 - y0).max(1.0);
    let steps = distance.ceil() as i32;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        fill_circle(image, x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, radius, color);
    }
}

fn fill_circle(image: &mut RgbaImage, cx: f32, cy: f32, radius: f32, color: Rgba<u8>) {
    let min_x = (cx - radius).floor() as i32;
    let max_x = (cx + radius).ceil() as i32;
    let min_y = (cy - radius).floor() as i32;
    let max_y = (cy + radius).ceil() as i32;
    let r2 = radius * radius;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r2 {
                blend_pixel(image, x, y, color, color[3] as f32 / 255.0);
            }
        }
    }
}

fn fill_rect(image: &mut RgbaImage, x: f32, y: f32, width: f32, height: f32, color: Rgba<u8>) {
    let min_x = x.max(0.0).floor() as u32;
    let min_y = y.max(0.0).floor() as u32;
    let max_x = (x + width).min(image.width() as f32).ceil() as u32;
    let max_y = (y + height).min(image.height() as f32).ceil() as u32;
    for y in min_y..max_y {
        for x in min_x..max_x {
            blend_pixel(image, x as i32, y as i32, color, color[3] as f32 / 255.0);
        }
    }
}

fn stroke_rect(image: &mut RgbaImage, x: f32, y: f32, width: f32, height: f32, color: Rgba<u8>) {
    draw_thick_line(image, x, y, x + width, y, 1.0, color);
    draw_thick_line(image, x + width, y, x + width, y + height, 1.0, color);
    draw_thick_line(image, x + width, y + height, x, y + height, 1.0, color);
    draw_thick_line(image, x, y + height, x, y, 1.0, color);
}

fn blend_pixel(image: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>, alpha: f32) {
    if x < 0 || y < 0 || x >= image.width() as i32 || y >= image.height() as i32 {
        return;
    }
    let pixel = image.get_pixel_mut(x as u32, y as u32);
    let alpha = alpha.clamp(0.0, 1.0) * (color[3] as f32 / 255.0);
    for channel in 0..3 {
        pixel[channel] =
            (pixel[channel] as f32 * (1.0 - alpha) + color[channel] as f32 * alpha).round() as u8;
    }
    pixel[3] = 255;
}

fn parse_hex_color(value: &str) -> Option<Rgba<u8>> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    Some(Rgba([
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
        255,
    ]))
}

fn color_with_alpha(mut color: Rgba<u8>, alpha: u8) -> Rgba<u8> {
    color[3] = alpha;
    color
}
