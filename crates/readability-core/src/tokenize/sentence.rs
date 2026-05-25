use super::abbreviations::is_abbreviation;

pub fn segment_sentences(text: &str) -> Vec<&str> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let mut sentences = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let ch = bytes[i] as char;

        if ch == '.' || ch == '?' || ch == '!' {
            // Check for repeated terminal punctuation (!!!, ???)
            let mut end_punct = i + 1;
            while end_punct < len && (bytes[end_punct] == b'.' || bytes[end_punct] == b'?' || bytes[end_punct] == b'!') {
                end_punct += 1;
            }

            // Skip closing quotes/parens after punctuation
            let mut after_punct = end_punct;
            while after_punct < len {
                let next = bytes[after_punct] as char;
                if next == '"' || next == '\'' || next == ')' || next == '\u{201D}' as u8 as char {
                    after_punct += 1;
                } else {
                    break;
                }
            }

            if ch == '.' && is_period_sentence_boundary(text, i) {
                let sentence = text[start..after_punct].trim();
                if !sentence.is_empty() {
                    sentences.push(sentence);
                }
                start = after_punct;
            } else if ch == '?' || ch == '!' {
                let sentence = text[start..after_punct].trim();
                if !sentence.is_empty() {
                    sentences.push(sentence);
                }
                start = after_punct;
            }

            i = after_punct;
        } else {
            i += text[i..].chars().next().map_or(1, |c| c.len_utf8());
        }
    }

    // Remaining text
    let remaining = text[start..].trim();
    if !remaining.is_empty() {
        sentences.push(remaining);
    }

    // Guarantee at least one sentence for non-empty text
    if sentences.is_empty() && !text.trim().is_empty() {
        sentences.push(text.trim());
    }

    sentences
}

fn is_period_sentence_boundary(text: &str, period_pos: usize) -> bool {
    let before = &text[..period_pos];

    // Get the word before the period
    let word_before = before
        .rsplit(|c: char| c.is_whitespace())
        .next()
        .unwrap_or("");

    // Check abbreviation
    if is_abbreviation(word_before) {
        return false;
    }

    // Check for initials (single letter + period): "F. Scott"
    let stripped = word_before.trim_end_matches('.');
    if stripped.len() == 1 && stripped.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
        return false;
    }

    // Check for acronyms with periods: "U.S.", "A.I."
    if is_dotted_acronym(word_before) {
        return false;
    }

    // Check for decimal numbers: "3.14"
    if stripped.chars().all(|c| c.is_ascii_digit() || c == ',') && !stripped.is_empty() {
        // Look ahead for digits
        let after = &text[period_pos + 1..];
        if after.starts_with(|c: char| c.is_ascii_digit()) {
            return false;
        }
    }

    // Check if followed by a lowercase letter (not a sentence start)
    let after = text[period_pos + 1..].trim_start();
    if let Some(next_char) = after.chars().next() {
        if next_char.is_ascii_lowercase() {
            return false;
        }
    }

    true
}

fn is_dotted_acronym(word: &str) -> bool {
    let parts: Vec<&str> = word.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .all(|p| p.len() == 1 && p.chars().all(|c| c.is_ascii_uppercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_sentences() {
        let sentences = segment_sentences("Hello world. How are you? I am fine!");
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn abbreviation_mr() {
        let sentences = segment_sentences("Mr. Smith went to Washington.");
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn abbreviation_dr() {
        let sentences = segment_sentences("Dr. Jones wrote e.g. examples.");
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn us_acronym() {
        let sentences = segment_sentences("The U.S. economy grew.");
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn decimal_number() {
        let sentences = segment_sentences("Version 3.14 is stable.");
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn repeated_punctuation() {
        let sentences = segment_sentences("Hello!!! Are you there???");
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn empty_text() {
        let sentences = segment_sentences("");
        assert_eq!(sentences.len(), 0);
    }

    #[test]
    fn no_punctuation_fallback() {
        let sentences = segment_sentences("This text has no ending punctuation");
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn single_initial() {
        let sentences = segment_sentences("F. Scott Fitzgerald wrote novels.");
        assert_eq!(sentences.len(), 1);
    }
}
