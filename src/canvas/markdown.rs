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
    let icon = match kind.as_str() {
        "NOTE" | "INFO" => "ℹ️",
        "TIP" | "HINT" | "IMPORTANT" => "💡",
        "WARNING" | "CAUTION" | "ATTENTION" => "⚠️",
        "ERROR" | "FAIL" | "FAILURE" | "DANGER" | "BUG" => "⛔",
        "QUESTION" | "HELP" | "FAQ" => "❓",
        "QUOTE" | "CITE" => "❝",
        "TODO" => "☐",
        "SUCCESS" | "CHECK" | "DONE" => "✅",
        _ => "▣",
    };

    if title.is_empty() {
        format!("{indent}> **{icon} {kind}**")
    } else {
        format!("{indent}> **{icon} {kind}: {title}**")
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
}
