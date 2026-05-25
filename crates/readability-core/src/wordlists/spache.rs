include!(concat!(env!("OUT_DIR"), "/spache_words.rs"));

pub fn is_spache_familiar(word: &str) -> bool {
    SPACHE_EASY_WORDS.contains(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_words_are_familiar() {
        assert!(is_spache_familiar("a"));
        assert!(is_spache_familiar("about"));
        assert!(is_spache_familiar("the"));
    }

    #[test]
    fn advanced_words_are_unfamiliar() {
        assert!(!is_spache_familiar("concatenation"));
        assert!(!is_spache_familiar("ubiquitous"));
    }
}
