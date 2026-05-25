use crate::stats::TextStats;
use readability_types::FormulaErrorCode;

pub fn automated_readability_index(stats: &TextStats) -> Result<f64, FormulaErrorCode> {
    if stats.words == 0 || stats.sentences == 0 {
        return Err(FormulaErrorCode::InsufficientText);
    }
    let chars_per_word = stats.ari_characters as f64 / stats.words as f64;
    let words_per_sentence = stats.words as f64 / stats.sentences as f64;
    Ok(4.71 * chars_per_word + 0.5 * words_per_sentence - 21.43)
}
