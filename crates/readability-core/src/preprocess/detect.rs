use readability_types::InputFormat;

pub fn detect_format(input: &str) -> InputFormat {
    let trimmed = input.trim_start();

    // Only check HTML tags if the string starts with ASCII '<'
    if trimmed.starts_with('<') {
        let lower_start: String = trimmed.chars().take(10).collect::<String>().to_ascii_lowercase();
        if lower_start.starts_with("<!doctype") || lower_start.starts_with("<html") {
            return InputFormat::Html;
        }
    }

    let check_slice: String = trimmed.chars().take(1000).collect();

    if check_slice.contains("<p>")
        || check_slice.contains("<p ")
        || check_slice.contains("<div>")
        || check_slice.contains("<div ")
    {
        return InputFormat::Html;
    }

    for i in 1..=6 {
        let tag = format!("<h{}", i);
        if check_slice.contains(&tag) {
            return InputFormat::Html;
        }
    }

    if trimmed.starts_with("---\n") || trimmed.starts_with("---\r\n") {
        return InputFormat::MarkdownLite;
    }

    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            return InputFormat::MarkdownLite;
        }
        break;
    }

    InputFormat::Plain
}
