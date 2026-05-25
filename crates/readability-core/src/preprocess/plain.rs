pub fn preprocess_plain(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut last_was_space = false;
    let mut consecutive_newlines = 0;

    for ch in input.chars() {
        let normalized = normalize_char(ch);

        match normalized {
            Some('\n') => {
                consecutive_newlines += 1;
                if consecutive_newlines == 2 {
                    result.push('\n');
                    result.push('\n');
                    last_was_space = false;
                }
            }
            Some(c) if c.is_whitespace() => {
                consecutive_newlines = 0;
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            }
            Some(c) => {
                consecutive_newlines = 0;
                result.push(c);
                last_was_space = false;
            }
            None => {}
        }
    }

    result.trim().to_string()
}

fn normalize_char(ch: char) -> Option<char> {
    match ch {
        // BOM
        '\u{FEFF}' => None,
        // Soft hyphen
        '\u{00AD}' => None,
        // Control characters (except useful whitespace)
        c if c.is_control() && c != '\n' && c != '\t' => None,
        // Typographic quotes -> ASCII
        '\u{2018}' | '\u{2019}' | '\u{201B}' => Some('\''),
        '\u{201C}' | '\u{201D}' | '\u{201F}' => Some('"'),
        // En dash, em dash -> hyphen
        '\u{2013}' | '\u{2014}' => Some('-'),
        // Ellipsis -> three periods
        '\u{2026}' => Some('.'),
        // Non-breaking space -> regular space
        '\u{00A0}' => Some(' '),
        // Tab -> space
        '\t' => Some(' '),
        // Everything else passes through
        c => Some(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_smart_quotes() {
        let input = "\u{201C}Hello,\u{201D} she said.";
        let result = preprocess_plain(input);
        assert_eq!(result, "\"Hello,\" she said.");
    }

    #[test]
    fn preserves_paragraph_boundaries() {
        let input = "First paragraph.\n\nSecond paragraph.";
        let result = preprocess_plain(input);
        assert!(result.contains("\n\n"));
    }

    #[test]
    fn collapses_whitespace() {
        let input = "Hello    world   test";
        let result = preprocess_plain(input);
        assert_eq!(result, "Hello world test");
    }

    #[test]
    fn strips_bom() {
        let input = "\u{FEFF}Hello world.";
        let result = preprocess_plain(input);
        assert_eq!(result, "Hello world.");
    }

    #[test]
    fn normalizes_ellipsis() {
        let input = "Wait\u{2026} what?";
        let result = preprocess_plain(input);
        assert_eq!(result, "Wait. what?");
    }
}
