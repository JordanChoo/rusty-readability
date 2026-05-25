use crate::stats::TextStats;
use readability_types::FormulaErrorCode;

pub fn flesch_kincaid_grade(stats: &TextStats) -> Result<f64, FormulaErrorCode> {
    if stats.words == 0 || stats.sentences == 0 {
        return Err(FormulaErrorCode::InsufficientText);
    }
    let syllables = stats.syllables.ok_or(FormulaErrorCode::Internal)?;
    let asl = stats.words as f64 / stats.sentences as f64;
    let asw = syllables as f64 / stats.words as f64;
    Ok(0.39 * asl + 11.8 * asw - 15.59)
}
