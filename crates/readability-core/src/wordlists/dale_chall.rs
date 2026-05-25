include!(concat!(env!("OUT_DIR"), "/dale_chall_words.rs"));

pub fn is_dale_chall_familiar(word: &str) -> bool {
    DALE_CHALL_EASY_WORDS.contains(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_words_are_familiar() {
        assert!(is_dale_chall_familiar("the"));
        assert!(is_dale_chall_familiar("about"));
        assert!(is_dale_chall_familiar("people"));
    }

    #[test]
    fn uncommon_words_are_unfamiliar() {
        assert!(!is_dale_chall_familiar("sesquipedalian"));
        assert!(!is_dale_chall_familiar("ameliorate"));
    }
}
