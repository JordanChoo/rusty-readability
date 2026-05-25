use crate::stats::TextStats;
use readability_types::FormulaErrorCode;

pub fn coleman_liau(stats: &TextStats) -> Result<f64, FormulaErrorCode> {
    if stats.words == 0 {
        return Err(FormulaErrorCode::InsufficientText);
    }
    let l100 = 100.0 * stats.letters_ascii as f64 / stats.words as f64;
    let s100 = 100.0 * stats.sentences as f64 / stats.words as f64;
    Ok(0.0588 * l100 - 0.296 * s100 - 15.8)
}
