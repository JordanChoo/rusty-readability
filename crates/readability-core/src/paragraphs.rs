use readability_types::{HardestSentence, ParagraphBreakdown};

use crate::difficulty::difficulty_label;
use crate::stats::SentenceInfo;

pub fn compute_paragraph_breakdown(
    sentence_infos: &[SentenceInfo<'_>],
    paragraph_count: usize,
) -> Vec<ParagraphBreakdown> {
    let mut breakdowns = Vec::with_capacity(paragraph_count);

    for para_idx in 0..paragraph_count {
        let para_sentences: Vec<&SentenceInfo<'_>> = sentence_infos
            .iter()
            .filter(|s| s.paragraph_index == para_idx)
            .collect();

        let sent_count = para_sentences.len();
        let word_count: usize = para_sentences.iter().map(|s| s.word_count).sum();

        let avg_sentence_length = if sent_count > 0 {
            word_count as f64 / sent_count as f64
        } else {
            0.0
        };

        let avg_syl = if !para_sentences.is_empty() {
            let total_spw: f64 = para_sentences.iter().map(|s| s.syllables_per_word).sum();
            Some(total_spw / para_sentences.len() as f64)
        } else {
            None
        };

        breakdowns.push(ParagraphBreakdown {
            index: para_idx,
            words: word_count,
            sentences: sent_count,
            avg_sentence_length,
            avg_syllables_per_word: avg_syl,
            difficulty: difficulty_label(avg_sentence_length).to_string(),
        });
    }

    breakdowns
}

pub fn find_hardest_sentences(
    sentence_infos: &[SentenceInfo<'_>],
    max_count: usize,
) -> Vec<HardestSentence> {
    if max_count == 0 || sentence_infos.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(usize, f64)> = sentence_infos
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let score = s.word_count as f64 * 0.6 + s.syllables_per_word * 20.0 + s.complex_word_count as f64 * 5.0;
            (i, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(max_count);

    scored
        .into_iter()
        .map(|(i, _)| {
            let s = &sentence_infos[i];
            let preview = if s.text.len() > 120 {
                let mut end = 117;
                while !s.text.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}...", &s.text[..end])
            } else {
                s.text.to_string()
            };

            HardestSentence {
                index: i,
                paragraph_index: s.paragraph_index,
                words: s.word_count,
                syllables_per_word: Some(s.syllables_per_word),
                complex_word_count: Some(s.complex_word_count),
                preview,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::SentenceInfo;

    #[test]
    fn paragraph_breakdown_single() {
        let infos = vec![
            SentenceInfo { text: "Hello world.", word_count: 2, syllables_per_word: 1.5, complex_word_count: 0, paragraph_index: 0 },
            SentenceInfo { text: "Bye now.", word_count: 2, syllables_per_word: 1.0, complex_word_count: 0, paragraph_index: 0 },
        ];
        let result = compute_paragraph_breakdown(&infos, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].words, 4);
        assert_eq!(result[0].sentences, 2);
    }

    #[test]
    fn hardest_sentences_ranking() {
        let infos = vec![
            SentenceInfo { text: "Short.", word_count: 1, syllables_per_word: 1.0, complex_word_count: 0, paragraph_index: 0 },
            SentenceInfo { text: "This is a considerably longer and more complex sentence with many words.", word_count: 12, syllables_per_word: 1.8, complex_word_count: 2, paragraph_index: 0 },
        ];
        let result = find_hardest_sentences(&infos, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 1);
    }
}
