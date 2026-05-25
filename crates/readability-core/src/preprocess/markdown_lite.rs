use regex_lite::Regex;

pub fn preprocess_markdown(input: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut in_code_block = false;
    let mut in_frontmatter = false;
    let mut frontmatter_started = false;

    for (i, line) in input.lines().enumerate() {
        if i == 0 && line.trim() == "---" {
            in_frontmatter = true;
            frontmatter_started = true;
            continue;
        }
        if in_frontmatter && frontmatter_started && line.trim() == "---" {
            in_frontmatter = false;
            continue;
        }
        if in_frontmatter {
            continue;
        }

        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        let processed = strip_markdown_syntax(line);
        lines.push(processed);
    }

    let joined = lines.join("\n");
    super::plain::preprocess_plain(&joined)
}

fn strip_markdown_syntax(line: &str) -> String {
    let trimmed = line.trim_start();

    // Headers: # through ######
    let heading_re = Regex::new(r"^#{1,6}\s+").unwrap();
    let stripped = heading_re.replace(trimmed, "").to_string();

    // List markers: -, *, +, or numbered
    let list_re = Regex::new(r"^(\s*[-*+]|\s*\d+\.)\s+").unwrap();
    let stripped = list_re.replace(&stripped, "").to_string();

    // Blockquotes
    let stripped = stripped.strip_prefix("> ").unwrap_or(&stripped).to_string();

    // Bold/italic
    let bold_re = Regex::new(r"\*\*(.+?)\*\*").unwrap();
    let stripped = bold_re.replace_all(&stripped, "$1").to_string();
    let italic_re = Regex::new(r"\*(.+?)\*").unwrap();
    let stripped = italic_re.replace_all(&stripped, "$1").to_string();

    // Inline code
    let code_re = Regex::new(r"`([^`]+)`").unwrap();
    let stripped = code_re.replace_all(&stripped, "$1").to_string();

    // Images: ![alt](url) -> alt (must run before links)
    let img_re = Regex::new(r"!\[([^\]]*)\]\([^)]+\)").unwrap();
    let stripped = img_re.replace_all(&stripped, "$1").to_string();

    // Links: [text](url) -> text
    let link_re = Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap();
    let stripped = link_re.replace_all(&stripped, "$1").to_string();

    stripped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_headers() {
        let md = "# Hello World\n\nThis is text.";
        let result = preprocess_markdown(md);
        assert!(result.contains("Hello World"));
        assert!(!result.contains("#"));
    }

    #[test]
    fn strips_bold_italic() {
        let md = "This is **bold** and *italic* text.";
        let result = preprocess_markdown(md);
        assert_eq!(result, "This is bold and italic text.");
    }

    #[test]
    fn skips_code_blocks() {
        let md = "Before.\n\n```rust\nlet x = 42;\n```\n\nAfter.";
        let result = preprocess_markdown(md);
        assert!(result.contains("Before."));
        assert!(result.contains("After."));
        assert!(!result.contains("let x"));
    }

    #[test]
    fn skips_frontmatter() {
        let md = "---\ntitle: Test\n---\n\nActual content.";
        let result = preprocess_markdown(md);
        assert!(result.contains("Actual content."));
        assert!(!result.contains("title"));
    }

    #[test]
    fn strips_links() {
        let md = "Click [here](https://example.com) for more.";
        let result = preprocess_markdown(md);
        assert_eq!(result, "Click here for more.");
    }

    #[test]
    fn strips_inline_code() {
        let md = "Use the `print` function.";
        let result = preprocess_markdown(md);
        assert_eq!(result, "Use the print function.");
    }
}
