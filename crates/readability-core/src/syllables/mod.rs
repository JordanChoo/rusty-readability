pub mod rules;
pub mod exceptions;
pub mod cache;

pub fn count_syllables(word: &str) -> u8 {
    let normalized = word.to_ascii_lowercase();

    if let Some(cached) = cache::cache_get(&normalized) {
        return cached;
    }

    let count = if let Some(exception) = exceptions::lookup_exception(&normalized) {
        exception
    } else {
        rules::count_syllables_rules(&normalized)
    };

    let count = count.max(1);
    cache::cache_put(normalized, count);
    count
}

pub fn is_polysyllabic(word: &str) -> bool {
    count_syllables(word) >= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_exception_for_business() {
        assert_eq!(count_syllables("business"), 2);
    }

    #[test]
    fn uses_rules_for_regular_word() {
        assert_eq!(count_syllables("reading"), 2);
    }

    #[test]
    fn polysyllabic_detection() {
        assert!(is_polysyllabic("international"));
        assert!(!is_polysyllabic("cat"));
    }

    #[test]
    fn minimum_one_syllable() {
        assert_eq!(count_syllables("a"), 1);
        assert_eq!(count_syllables("x"), 1);
    }
}
