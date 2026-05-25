use regex_lite::Regex;

pub fn preprocess_html(input: &str) -> String {
    let script_re = Regex::new(r"(?si)<script[^>]*>.*?</script>").unwrap();
    let style_re = Regex::new(r"(?si)<style[^>]*>.*?</style>").unwrap();

    let no_scripts = script_re.replace_all(input, "");
    let no_styles = style_re.replace_all(&no_scripts, "");

    let block_re = Regex::new(r"(?i)</?(p|div|br|h[1-6]|li|tr|blockquote|pre|hr|section|article|header|footer|aside|main|nav|figure|figcaption|details|summary)[^>]*>").unwrap();
    let with_breaks = block_re.replace_all(&no_styles, "\n");

    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let no_tags = tag_re.replace_all(&with_breaks, "");

    let entity_decoded = decode_entities(&no_tags);

    super::plain::preprocess_plain(&entity_decoded)
}

fn decode_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tags() {
        let html = "<p>Hello <b>world</b>.</p>";
        let result = preprocess_html(html);
        assert_eq!(result, "Hello world.");
    }

    #[test]
    fn removes_scripts() {
        let html = "Before. <script>alert('xss')</script> After.";
        let result = preprocess_html(html);
        assert!(result.contains("Before."));
        assert!(result.contains("After."));
        assert!(!result.contains("alert"));
    }

    #[test]
    fn removes_styles() {
        let html = "Text. <style>body { color: red; }</style> More text.";
        let result = preprocess_html(html);
        assert!(!result.contains("color"));
    }

    #[test]
    fn decodes_entities() {
        let html = "Tom &amp; Jerry &lt;friends&gt;";
        let result = preprocess_html(html);
        assert_eq!(result, "Tom & Jerry <friends>");
    }

    #[test]
    fn block_elements_become_paragraphs() {
        let html = "<p>First paragraph.</p><p>Second paragraph.</p>";
        let result = preprocess_html(html);
        assert!(result.contains("First paragraph."));
        assert!(result.contains("Second paragraph."));
    }
}
