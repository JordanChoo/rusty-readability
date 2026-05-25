pub struct WordToken<'a> {
    pub text: &'a str,
    pub lowercase: String,
    pub has_letter: bool,
    pub is_numeric: bool,
}

pub fn tokenize_words(text: &str) -> Vec<WordToken<'_>> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (i, ch) in text.char_indices() {
        if is_word_char(ch) {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start {
            let end = i;
            let word = &text[s..end];
            if let Some(token) = make_token(word) {
                tokens.push(token);
            }
            start = None;
        }
    }

    // Handle last word
    if let Some(s) = start {
        let word = &text[s..];
        if let Some(token) = make_token(word) {
            tokens.push(token);
        }
    }

    tokens
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '\'' || ch == '\u{2019}' || ch == '-'
}

fn make_token(word: &str) -> Option<WordToken<'_>> {
    // Trim leading/trailing punctuation from the token
    let trimmed = word.trim_matches(|c: char| c == '\'' || c == '\u{2019}' || c == '-');
    if trimmed.is_empty() {
        return None;
    }

    let has_letter = trimmed.chars().any(|c| c.is_alphabetic());

    // A word must contain at least one letter
    if !has_letter {
        return None;
    }

    let is_numeric = trimmed.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',');

    // Normalize curly apostrophes
    let lowercase = trimmed
        .replace('\u{2019}', "'")
        .to_lowercase();

    Some(WordToken {
        text: trimmed,
        lowercase,
        has_letter,
        is_numeric,
    })
}

pub fn count_letters_ascii(word: &str) -> usize {
    word.chars().filter(|c| c.is_ascii_alphabetic()).count()
}

pub fn count_ari_characters(word: &str) -> usize {
    word.chars().filter(|c| c.is_ascii_alphanumeric()).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_words() {
        let tokens = tokenize_words("Hello world test");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].text, "Hello");
        assert_eq!(tokens[0].lowercase, "hello");
    }

    #[test]
    fn contractions() {
        let tokens = tokenize_words("don't can't we're");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].lowercase, "don't");
    }

    #[test]
    fn hyphenated() {
        let tokens = tokenize_words("well-known state-of-the-art");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn strips_pure_numbers() {
        let tokens = tokenize_words("42 and 100");
        // Pure numbers without letters are excluded
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "and");
    }

    #[test]
    fn curly_apostrophe() {
        let tokens = tokenize_words("don\u{2019}t");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].lowercase, "don't");
    }

    #[test]
    fn preserves_casing() {
        let tokens = tokenize_words("Smith");
        assert_eq!(tokens[0].text, "Smith");
        assert_eq!(tokens[0].lowercase, "smith");
    }
}
