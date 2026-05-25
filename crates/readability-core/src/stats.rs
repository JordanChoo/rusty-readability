use std::collections::HashSet;

use crate::syllables;
use crate::tokenize::sentence::segment_sentences;
use crate::tokenize::word::{count_ari_characters, count_letters_ascii, tokenize_words, WordToken};
use crate::wordlists;

pub struct TextStats {
    // Tier 0: always computed
    pub bytes: usize,
    pub graphemes: usize,
    pub letters_ascii: usize,
    pub letters_unicode: usize,
    pub ari_characters: usize,
    pub words: usize,
    pub unique_words: usize,
    pub hapax_count: usize,
    pub sentences: usize,
    pub paragraphs: usize,
    pub average_words_per_sentence: f64,
    pub average_characters_per_word: f64,
    pub longest_word_len: usize,
    pub longest_sentence_words: usize,

    // Tier 1: syllable-dependent
    pub syllables: Option<usize>,
    pub polysyllables: Option<usize>,
    pub complex_words: Option<usize>,
    pub complex_words_excluding_proper: Option<usize>,
    pub average_syllables_per_word: Option<f64>,
    pub syllable_estimates_from_dictionary: Option<usize>,
    pub syllable_estimates_from_rules: Option<usize>,

    // Tier 2: word-list-dependent
    pub dale_chall_difficult_words: Option<usize>,
    pub dale_chall_difficult_percentage: Option<f64>,
    pub spache_unique_unfamiliar_words: Option<usize>,
    pub spache_unique_unfamiliar_percentage: Option<f64>,
}

pub struct SentenceInfo<'a> {
    pub text: &'a str,
    pub word_count: usize,
    pub syllables_per_word: f64,
    pub complex_word_count: usize,
    pub paragraph_index: usize,
}

pub struct ComputeResult<'a> {
    pub stats: TextStats,
    pub sentence_infos: Vec<SentenceInfo<'a>>,
    pub paragraph_boundaries: Vec<usize>,
}

pub fn compute_stats(text: &str) -> ComputeResult<'_> {
    let bytes = text.len();
    let graphemes = text.chars().count();

    let paragraphs_split: Vec<&str> = split_paragraphs(text);
    let paragraph_count = paragraphs_split.len().max(1);

    let sentences = segment_sentences(text);
    let sentence_count = sentences.len().max(1);

    let mut all_tokens: Vec<WordToken<'_>> = Vec::new();
    let mut word_freq: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut total_letters_ascii: usize = 0;
    let mut total_letters_unicode: usize = 0;
    let mut total_ari_chars: usize = 0;
    let mut longest_word_len: usize = 0;
    let mut longest_sentence_words: usize = 0;

    // Tier 1 accumulators — computed in one pass alongside Tier 0
    let mut total_syllables: usize = 0;
    let mut total_polysyllables: usize = 0;
    let mut total_complex: usize = 0;
    let mut total_complex_excl_proper: usize = 0;

    let mut sentence_infos: Vec<SentenceInfo<'_>> = Vec::new();
    let mut paragraph_boundaries: Vec<usize> = Vec::new();

    let mut current_para_idx = 0;
    for (sent_idx, &sent_text) in sentences.iter().enumerate() {
        let tokens = tokenize_words(sent_text);
        let word_count = tokens.len();

        if word_count > longest_sentence_words {
            longest_sentence_words = word_count;
        }

        while current_para_idx + 1 < paragraphs_split.len() {
            let para_end_byte = paragraph_byte_end(text, &paragraphs_split, current_para_idx);
            let sent_start = sent_text.as_ptr() as usize - text.as_ptr() as usize;
            if sent_start >= para_end_byte {
                current_para_idx += 1;
            } else {
                break;
            }
        }

        if sent_idx == 0 || (sent_idx > 0 && sentence_infos.last().map(|s| s.paragraph_index) != Some(current_para_idx)) {
            paragraph_boundaries.push(sent_idx);
        }

        let mut sent_syllables: usize = 0;
        let mut sent_complex: usize = 0;

        for token in &tokens {
            let syl = syllables::count_syllables(&token.lowercase) as usize;
            sent_syllables += syl;
            total_syllables += syl;

            if syl >= 3 {
                sent_complex += 1;
                total_polysyllables += 1;
                total_complex += 1;
                let first_char = token.text.chars().next();
                let is_proper = first_char.map_or(false, |c| c.is_uppercase());
                if !is_proper {
                    total_complex_excl_proper += 1;
                }
            }

            let letters_a = count_letters_ascii(token.text);
            let letters_u = token.text.chars().filter(|c| c.is_alphabetic()).count();
            let ari_c = count_ari_characters(token.text);

            total_letters_ascii += letters_a;
            total_letters_unicode += letters_u;
            total_ari_chars += ari_c;

            if letters_a > longest_word_len {
                longest_word_len = letters_a;
            }

            *word_freq.entry(token.text).or_insert(0) += 1;
        }

        let spw = if word_count > 0 {
            sent_syllables as f64 / word_count as f64
        } else {
            0.0
        };

        sentence_infos.push(SentenceInfo {
            text: sent_text,
            word_count,
            syllables_per_word: spw,
            complex_word_count: sent_complex,
            paragraph_index: current_para_idx,
        });

        all_tokens.extend(tokens);
    }

    let word_count = all_tokens.len();
    let unique_words = word_freq.len();
    let hapax_count = word_freq.values().filter(|&&c| c == 1).count();

    let avg_words_per_sentence = if sentence_count > 0 {
        word_count as f64 / sentence_count as f64
    } else {
        0.0
    };
    let avg_chars_per_word = if word_count > 0 {
        total_letters_ascii as f64 / word_count as f64
    } else {
        0.0
    };

    let avg_syl_per_word = if word_count > 0 {
        Some(total_syllables as f64 / word_count as f64)
    } else {
        None
    };

    // Tier 2: word-list stats
    let mut dale_chall_difficult = 0usize;
    let mut spache_seen: HashSet<String> = HashSet::new();
    let mut spache_unfamiliar_count: usize = 0;

    for token in &all_tokens {
        let lc = &token.lowercase;
        let base = stem_simple(lc);

        if !wordlists::dale_chall::is_dale_chall_familiar(lc)
            && !wordlists::dale_chall::is_dale_chall_familiar(&base)
        {
            dale_chall_difficult += 1;
        }

        if !spache_seen.contains(lc.as_str()) {
            spache_seen.insert(lc.clone());
            if !wordlists::spache::is_spache_familiar(lc)
                && !wordlists::spache::is_spache_familiar(&base)
            {
                spache_unfamiliar_count += 1;
            }
        }
    }

    let dale_chall_pct = if word_count > 0 {
        100.0 * dale_chall_difficult as f64 / word_count as f64
    } else {
        0.0
    };
    let spache_unfam_pct = if unique_words > 0 {
        100.0 * spache_unfamiliar_count as f64 / unique_words as f64
    } else {
        0.0
    };

    let stats = TextStats {
        bytes,
        graphemes,
        letters_ascii: total_letters_ascii,
        letters_unicode: total_letters_unicode,
        ari_characters: total_ari_chars,
        words: word_count,
        unique_words,
        hapax_count,
        sentences: sentences.len(),
        paragraphs: paragraph_count,
        average_words_per_sentence: avg_words_per_sentence,
        average_characters_per_word: avg_chars_per_word,
        longest_word_len,
        longest_sentence_words,

        syllables: Some(total_syllables),
        polysyllables: Some(total_polysyllables),
        complex_words: Some(total_complex),
        complex_words_excluding_proper: Some(total_complex_excl_proper),
        average_syllables_per_word: avg_syl_per_word,
        syllable_estimates_from_dictionary: None,
        syllable_estimates_from_rules: None,

        dale_chall_difficult_words: Some(dale_chall_difficult),
        dale_chall_difficult_percentage: Some(dale_chall_pct),
        spache_unique_unfamiliar_words: Some(spache_unfamiliar_count),
        spache_unique_unfamiliar_percentage: Some(spache_unfam_pct),
    };

    ComputeResult {
        stats,
        sentence_infos,
        paragraph_boundaries,
    }
}

fn split_paragraphs(text: &str) -> Vec<&str> {
    let mut paragraphs = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();

    while i < len {
        if bytes[i] == b'\n' && i + 1 < len && bytes[i + 1] == b'\n' {
            let para = text[start..i].trim();
            if !para.is_empty() {
                paragraphs.push(para);
            }
            while i < len && bytes[i] == b'\n' {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }

    let remaining = text[start..].trim();
    if !remaining.is_empty() {
        paragraphs.push(remaining);
    }

    if paragraphs.is_empty() && !text.trim().is_empty() {
        paragraphs.push(text.trim());
    }

    paragraphs
}

fn paragraph_byte_end(full_text: &str, paragraphs: &[&str], idx: usize) -> usize {
    let para = paragraphs[idx];
    let para_start = para.as_ptr() as usize - full_text.as_ptr() as usize;
    para_start + para.len()
}

fn stem_simple(word: &str) -> String {
    if let Some(base) = word.strip_suffix("ing") {
        if base.len() >= 3 {
            return base.to_string();
        }
    }
    if let Some(base) = word.strip_suffix("ed") {
        if base.len() >= 3 {
            return base.to_string();
        }
    }
    if let Some(base) = word.strip_suffix("es") {
        if base.len() >= 3 {
            return base.to_string();
        }
    }
    if let Some(base) = word.strip_suffix("s") {
        if base.len() >= 3 {
            return base.to_string();
        }
    }
    if let Some(base) = word.strip_suffix("ly") {
        if base.len() >= 3 {
            return base.to_string();
        }
    }
    word.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_stats() {
        let result = compute_stats("The cat sat on the mat. The dog ran.");
        assert_eq!(result.stats.sentences, 2);
        assert!(result.stats.words > 0);
        assert!(result.stats.syllables.unwrap() > 0);
    }

    #[test]
    fn paragraph_detection() {
        let result = compute_stats("First paragraph here.\n\nSecond paragraph here.");
        assert_eq!(result.stats.paragraphs, 2);
    }

    #[test]
    fn syllable_stats_populated() {
        let result = compute_stats("The international community gathered.");
        assert!(result.stats.syllables.unwrap() > 0);
        assert!(result.stats.polysyllables.unwrap() > 0);
    }

    #[test]
    fn word_list_stats_populated() {
        let result = compute_stats("The concatenation of sesquipedalian verbiage.");
        assert!(result.stats.dale_chall_difficult_words.unwrap() > 0);
    }

    #[test]
    fn empty_text() {
        let result = compute_stats("");
        assert_eq!(result.stats.words, 0);
        assert_eq!(result.stats.sentences, 0);
    }

    #[test]
    fn stem_simple_works() {
        assert_eq!(stem_simple("running"), "runn");
        assert_eq!(stem_simple("walked"), "walk");
        assert_eq!(stem_simple("boxes"), "box");
        assert_eq!(stem_simple("cats"), "cat");
        assert_eq!(stem_simple("quickly"), "quick");
    }
}
