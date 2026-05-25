pub fn count_syllables_rules(word: &str) -> u8 {
    let w = word.to_ascii_lowercase();

    if w.is_empty() {
        return 1;
    }

    let chars: Vec<char> = w.chars().collect();
    let len = chars.len();

    if len <= 3 {
        return 1;
    }

    let mut count: i32 = 0;
    let mut prev_vowel = false;

    for (i, &ch) in chars.iter().enumerate() {
        let is_vowel = is_vowel_char(ch, i, &chars);

        if is_vowel && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_vowel;
    }

    // Silent-e: subtract 1 for trailing 'e' that isn't part of '-le'
    if chars.last() == Some(&'e') {
        if len >= 3 {
            let prev = chars[len - 2];
            // Keep syllable for terminal "-le" preceded by consonant (ta-ble, lit-tle)
            if prev == 'l' && len >= 3 && !is_base_vowel(chars[len - 3]) {
                // -le counts as a syllable, already counted by vowel-group
            } else if prev != 'l' || (len >= 3 && is_base_vowel(chars[len - 3])) {
                // Silent e
                count -= 1;
            }
        }
    }

    // Adjustments for -ed endings
    if w.ends_with("ed") && len > 3 {
        let before_ed = chars[len - 3];
        // -ed is silent except after 't' or 'd'
        if before_ed != 't' && before_ed != 'd' {
            count -= 1;
        }
    }

    // Adjustments for -es endings
    if w.ends_with("es") && len > 3 {
        let before_es = chars[len - 3];
        // -es adds syllable after sibilants: s, z, x, sh, ch
        if before_es != 's'
            && before_es != 'z'
            && before_es != 'x'
            && !(before_es == 'h' && len > 4 && (chars[len - 4] == 's' || chars[len - 4] == 'c'))
        {
            count -= 1;
        }
    }

    // Common diphthongs that may have been double-counted
    let diphthongs = ["ai", "au", "ay", "ea", "ee", "ei", "ey", "ie", "oa", "oo", "ou", "oy", "ue", "ui"];
    for d in &diphthongs {
        if w.contains(d) {
            // Already handled by vowel-group counting, but some patterns
            // might need adjustment. The vowel-group approach naturally
            // handles most diphthongs since they appear as one group.
        }
    }
    let _ = diphthongs;

    count.max(1) as u8
}

fn is_base_vowel(ch: char) -> bool {
    matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u')
}

fn is_vowel_char(ch: char, pos: usize, chars: &[char]) -> bool {
    match ch {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        'y' => {
            // 'y' is a vowel when not at the start or when between consonants
            if pos == 0 {
                false
            } else {
                let prev = chars[pos - 1];
                !is_base_vowel(prev)
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_words() {
        assert_eq!(count_syllables_rules("cat"), 1);
        assert_eq!(count_syllables_rules("dog"), 1);
        assert_eq!(count_syllables_rules("the"), 1);
    }

    #[test]
    fn two_syllable_words() {
        assert_eq!(count_syllables_rules("reading"), 2);
        assert_eq!(count_syllables_rules("happy"), 2);
        assert_eq!(count_syllables_rules("water"), 2);
    }

    #[test]
    fn silent_e() {
        assert_eq!(count_syllables_rules("make"), 1);
        assert_eq!(count_syllables_rules("time"), 1);
        assert_eq!(count_syllables_rules("cake"), 1);
    }

    #[test]
    fn terminal_le() {
        assert_eq!(count_syllables_rules("table"), 2);
        assert_eq!(count_syllables_rules("little"), 2);
        assert_eq!(count_syllables_rules("simple"), 2);
    }

    #[test]
    fn polysyllabic() {
        assert_eq!(count_syllables_rules("computer"), 3);
        assert_eq!(count_syllables_rules("international"), 5);
    }

    #[test]
    fn minimum_one() {
        assert_eq!(count_syllables_rules("a"), 1);
        assert_eq!(count_syllables_rules("I"), 1);
        assert_eq!(count_syllables_rules("the"), 1);
    }
}
