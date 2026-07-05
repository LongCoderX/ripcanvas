#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownBlock {
    pub kind: MarkdownBlockKind,
    pub text: String,
    pub plain: String,
    pub marker: String,
    pub level: u8,
    pub checked: bool,
    pub indent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownBlockKind {
    Paragraph,
    Heading,
    Task,
    List,
    Quote,
    Callout,
    Code,
    Embed,
    Rule,
    Table,
    BlockId,
}

impl MarkdownBlockKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Heading => "heading",
            Self::Task => "task",
            Self::List => "list",
            Self::Quote => "quote",
            Self::Callout => "callout",
            Self::Code => "code",
            Self::Embed => "embed",
            Self::Rule => "rule",
            Self::Table => "table",
            Self::BlockId => "block-id",
        }
    }
}

/// Convert Obsidian-flavored Markdown into a preview-friendly Markdown subset.
///
/// Slint renders CommonMark-style Markdown. Obsidian adds constructs such as
/// wikilinks, embeds, callouts, task lists, comments, tags, and block ids. This
/// normalizer keeps standard Markdown intact while translating Obsidian-only
/// syntax into readable preview text.
pub fn obsidian_markdown_preview(markdown: &str) -> String {
    let without_comments = strip_obsidian_comments(markdown);
    let mut output = String::new();

    for line in without_comments.lines() {
        let line = normalize_task_marker(line);
        let line = normalize_callout_line(&line);
        let line = normalize_embeds_and_links(&line);
        let line = normalize_highlights(&line);
        let line = normalize_block_id(&line);
        output.push_str(&line);
        output.push('\n');
    }

    if markdown.ends_with('\n') {
        output
    } else {
        output.trim_end_matches('\n').to_owned()
    }
}

pub fn obsidian_markdown_blocks(markdown: &str) -> Vec<MarkdownBlock> {
    let without_comments = strip_obsidian_comments(markdown);
    let lines: Vec<&str> = without_comments.lines().collect();
    let mut blocks = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if is_code_fence(trimmed) {
            let fence = &trimmed[..3];
            let mut code = Vec::new();
            index += 1;
            while index < lines.len() && !lines[index].trim_start().starts_with(fence) {
                code.push(lines[index]);
                index += 1;
            }
            if index < lines.len() {
                index += 1;
            }
            blocks.push(block(
                MarkdownBlockKind::Code,
                code.join("\n"),
                String::new(),
                0,
                false,
                0,
            ));
            continue;
        }

        if let Some((level, text)) = parse_heading(trimmed) {
            blocks.push(block(
                MarkdownBlockKind::Heading,
                normalize_inline(text),
                String::new(),
                level,
                false,
                0,
            ));
            index += 1;
            continue;
        }

        if is_rule(trimmed) {
            blocks.push(block(
                MarkdownBlockKind::Rule,
                String::new(),
                String::new(),
                0,
                false,
                0,
            ));
            index += 1;
            continue;
        }

        if let Some(text) = parse_callout(trimmed) {
            blocks.push(block(
                MarkdownBlockKind::Callout,
                normalize_inline(&text),
                String::new(),
                0,
                false,
                0,
            ));
            index += 1;
            continue;
        }

        if let Some(text) = trimmed.strip_prefix('>') {
            blocks.push(block(
                MarkdownBlockKind::Quote,
                normalize_inline(text.trim()),
                String::new(),
                0,
                false,
                0,
            ));
            index += 1;
            continue;
        }

        if let Some((checked, text, indent)) = parse_task(line) {
            blocks.push(block(
                MarkdownBlockKind::Task,
                normalize_inline(text),
                String::new(),
                0,
                checked,
                indent,
            ));
            index += 1;
            continue;
        }

        if let Some((marker, text, indent)) = parse_list_item(line) {
            blocks.push(block(
                MarkdownBlockKind::List,
                normalize_inline(text),
                marker,
                0,
                false,
                indent,
            ));
            index += 1;
            continue;
        }

        if is_embed_line(trimmed) {
            blocks.push(block(
                MarkdownBlockKind::Embed,
                embed_label(
                    trimmed
                        .trim_start_matches('!')
                        .trim_matches(&['[', ']'][..]),
                ),
                String::new(),
                0,
                false,
                0,
            ));
            index += 1;
            continue;
        }

        if is_block_id_line(trimmed) {
            blocks.push(block(
                MarkdownBlockKind::BlockId,
                trimmed.to_owned(),
                String::new(),
                0,
                false,
                0,
            ));
            index += 1;
            continue;
        }

        if is_table_line(trimmed) {
            let mut table = Vec::new();
            while index < lines.len() && is_table_line(lines[index].trim()) {
                table.push(lines[index].trim());
                index += 1;
            }
            blocks.push(block(
                MarkdownBlockKind::Table,
                table.join("\n"),
                String::new(),
                0,
                false,
                0,
            ));
            continue;
        }

        let mut paragraph = vec![trimmed];
        index += 1;
        while index < lines.len() {
            let candidate = lines[index].trim();
            if candidate.is_empty() || starts_block(candidate) || parse_task(lines[index]).is_some()
            {
                break;
            }
            paragraph.push(candidate);
            index += 1;
        }
        blocks.push(block(
            MarkdownBlockKind::Paragraph,
            normalize_inline(&paragraph.join(" ")),
            String::new(),
            0,
            false,
            0,
        ));
    }

    blocks
}

fn block(
    kind: MarkdownBlockKind,
    text: String,
    marker: String,
    level: u8,
    checked: bool,
    indent: u8,
) -> MarkdownBlock {
    MarkdownBlock {
        kind,
        plain: text.clone(),
        text,
        marker,
        level,
        checked,
        indent,
    }
}

fn normalize_inline(text: &str) -> String {
    normalize_highlights(&normalize_embeds_and_links(text))
}

fn strip_obsidian_comments(markdown: &str) -> String {
    let mut output = String::new();
    let mut rest = markdown;
    while let Some(start) = rest.find("%%") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("%%") else {
            break;
        };
        rest = &after_start[end + 2..];
    }
    output.push_str(rest);
    output
}

fn normalize_task_marker(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    for marker in ["- [ ] ", "* [ ] ", "+ [ ] "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return format!("{indent}- ☐ {rest}");
        }
    }
    for marker in ["- [x] ", "- [X] ", "* [x] ", "* [X] ", "+ [x] ", "+ [X] "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return format!("{indent}- ☑ {rest}");
        }
    }
    line.to_owned()
}

fn parse_task(line: &str) -> Option<(bool, &str, u8)> {
    let trimmed = line.trim_start();
    let indent = ((line.len() - trimmed.len()) / 2).min(u8::MAX as usize) as u8;
    for marker in ["- [ ] ", "* [ ] ", "+ [ ] "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return Some((false, rest, indent));
        }
    }
    for marker in ["- [x] ", "- [X] ", "* [x] ", "* [X] ", "+ [x] ", "+ [X] "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return Some((true, rest, indent));
        }
    }
    None
}

fn parse_list_item(line: &str) -> Option<(String, &str, u8)> {
    let trimmed = line.trim_start();
    let indent = ((line.len() - trimmed.len()) / 2).min(u8::MAX as usize) as u8;
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return Some(("•".to_owned(), rest, indent));
        }
    }

    let dot = trimmed.find('.')?;
    let marker = &trimmed[..dot];
    if marker.is_empty() || !marker.chars().all(|char| char.is_ascii_digit()) {
        return None;
    }
    let rest = trimmed[dot + 1..].strip_prefix(' ')?;
    Some((format!("{marker}."), rest, indent))
}

fn parse_heading(line: &str) -> Option<(u8, &str)> {
    let level = line.chars().take_while(|char| *char == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let text = line[level..].strip_prefix(' ')?;
    Some((level as u8, text.trim()))
}

fn parse_callout(line: &str) -> Option<String> {
    let rest = line.strip_prefix("> [!")?;
    let end = rest.find(']')?;
    let kind = rest[..end].trim().to_uppercase();
    let title = rest[end + 1..].trim();
    let icon = callout_icon(&kind);
    if title.is_empty() {
        Some(format!("{icon} {kind}"))
    } else {
        Some(format!("{icon} {kind}: {title}"))
    }
}

fn normalize_callout_line(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent = &line[..line.len() - trimmed.len()];
    let Some(rest) = trimmed.strip_prefix("> [!") else {
        return line.to_owned();
    };
    let Some(end) = rest.find(']') else {
        return line.to_owned();
    };
    let kind = rest[..end].trim().to_uppercase();
    let title = rest[end + 1..].trim();
    let icon = callout_icon(&kind);

    if title.is_empty() {
        format!("{indent}> **{icon} {kind}**")
    } else {
        format!("{indent}> **{icon} {kind}: {title}**")
    }
}

fn callout_icon(kind: &str) -> &'static str {
    match kind {
        "NOTE" | "INFO" => "ℹ️",
        "TIP" | "HINT" | "IMPORTANT" => "💡",
        "WARNING" | "CAUTION" | "ATTENTION" => "⚠️",
        "ERROR" | "FAIL" | "FAILURE" | "DANGER" | "BUG" => "⛔",
        "QUESTION" | "HELP" | "FAQ" => "❓",
        "QUOTE" | "CITE" => "❝",
        "TODO" => "☐",
        "SUCCESS" | "CHECK" | "DONE" => "✅",
        _ => "▣",
    }
}

fn normalize_embeds_and_links(line: &str) -> String {
    let mut output = String::new();
    let mut rest = line;

    while let Some(start) = rest.find("[[") {
        output.push_str(&rest[..start]);
        let before = &rest[..start];
        let is_embed = before.ends_with('!');
        if is_embed {
            output.pop();
        }

        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("]]") else {
            output.push_str(&rest[start..]);
            return output;
        };

        let target = &after_start[..end];
        let replacement = if is_embed {
            format!("📎 {}", embed_label(target))
        } else {
            let label = wikilink_label(target);
            let destination = target
                .split_once('|')
                .map(|(destination, _)| destination)
                .unwrap_or(target)
                .replace(' ', "%20");
            format!("[{label}]({destination})")
        };
        output.push_str(&replacement);
        rest = &after_start[end + 2..];
    }

    output.push_str(rest);
    output
}

fn embed_label(target: &str) -> String {
    let target_without_size = target
        .split_once('|')
        .map(|(target, _)| target)
        .unwrap_or(target)
        .trim();
    let label = target_without_size.replace('#', " › ").replace('^', " #");
    if label.is_empty() {
        target.trim().to_owned()
    } else {
        label
    }
}

fn wikilink_label(target: &str) -> String {
    let visible = target
        .split_once('|')
        .map(|(_, alias)| alias)
        .unwrap_or(target)
        .split_once('#')
        .map(|(_, heading)| heading)
        .unwrap_or_else(|| {
            target
                .split_once('|')
                .map(|(_, alias)| alias)
                .unwrap_or(target)
        })
        .trim();

    if visible.is_empty() {
        target.trim().to_owned()
    } else {
        visible.to_owned()
    }
}

fn normalize_highlights(line: &str) -> String {
    let mut output = String::new();
    let mut rest = line;
    let mut open = false;

    while let Some(index) = rest.find("==") {
        output.push_str(&rest[..index]);
        output.push_str("**");
        open = !open;
        rest = &rest[index + 2..];
    }
    output.push_str(rest);

    if open {
        output.push_str("**");
    }
    output
}

fn normalize_block_id(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with('^') && trimmed[1..].chars().all(is_block_id_char) {
        format!("`{trimmed}`")
    } else {
        line.to_owned()
    }
}

fn starts_block(line: &str) -> bool {
    is_code_fence(line)
        || parse_heading(line).is_some()
        || is_rule(line)
        || parse_callout(line).is_some()
        || line.starts_with('>')
        || is_embed_line(line)
        || is_block_id_line(line)
        || is_table_line(line)
}

fn is_code_fence(line: &str) -> bool {
    line.starts_with("```") || line.starts_with("~~~")
}

fn is_rule(line: &str) -> bool {
    matches!(line, "---" | "***" | "___")
}

fn is_embed_line(line: &str) -> bool {
    line.starts_with("![[") && line.ends_with("]]")
}

fn is_block_id_line(line: &str) -> bool {
    line.starts_with('^') && line[1..].chars().all(is_block_id_char)
}

fn is_table_line(line: &str) -> bool {
    line.starts_with('|') && line.ends_with('|') && line.matches('|').count() >= 2
}

fn is_block_id_char(char: char) -> bool {
    char.is_ascii_alphanumeric() || char == '-' || char == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_obsidian_wikilinks_embeds_tasks_and_callouts() {
        let markdown = "\
> [!note] Read this
- [ ] Todo [[Plan|planning note]]
- [x] Done ![[diagram.png|300]]
==Marked==
%% hidden %%
^block-id";

        let preview = obsidian_markdown_preview(markdown);

        assert!(preview.contains("> **ℹ️ NOTE: Read this**"));
        assert!(preview.contains("- ☐ Todo [planning note](Plan)"));
        assert!(preview.contains("- ☑ Done 📎 diagram.png"));
        assert!(preview.contains("**Marked**"));
        assert!(!preview.contains("hidden"));
        assert!(preview.contains("`^block-id`"));
    }

    #[test]
    fn parses_obsidian_markdown_into_renderable_blocks() {
        let markdown = "\
# Heading
Intro with [[Note|alias]] and ==mark==.

> [!warning] Careful
> quoted

- [ ] Task
1. Ordered
![[image.png|300]]

```rust
fn main() {}
```

| A | B |
| - | - |
| 1 | 2 |
^block";

        let blocks = obsidian_markdown_blocks(markdown);

        assert_eq!(blocks[0].kind, MarkdownBlockKind::Heading);
        assert_eq!(blocks[0].level, 1);
        assert_eq!(blocks[1].kind, MarkdownBlockKind::Paragraph);
        assert!(blocks[1].text.contains("[alias](Note)"));
        assert!(blocks[1].text.contains("**mark**"));
        assert_eq!(blocks[2].kind, MarkdownBlockKind::Callout);
        assert_eq!(blocks[3].kind, MarkdownBlockKind::Quote);
        assert_eq!(blocks[4].kind, MarkdownBlockKind::Task);
        assert!(!blocks[4].checked);
        assert_eq!(blocks[5].kind, MarkdownBlockKind::List);
        assert_eq!(blocks[5].marker, "1.");
        assert_eq!(blocks[6].kind, MarkdownBlockKind::Embed);
        assert_eq!(blocks[7].kind, MarkdownBlockKind::Code);
        assert_eq!(blocks[8].kind, MarkdownBlockKind::Table);
        assert_eq!(blocks[9].kind, MarkdownBlockKind::BlockId);
    }
}
