use crate::stats::TextStats;
use readability_types::FormulaErrorCode;

/// SMOG Index (McLaughlin, 1969)
/// CRITICAL: 3.1291 is ADDED outside the multiplication by 1.043
/// Canonical: 1.043 * sqrt(polysyllables * 30 / sentences) + 3.1291
pub fn smog(stats: &TextStats) -> Result<f64, FormulaErrorCode> {
    if stats.sentences == 0 {
        return Err(FormulaErrorCode::InsufficientText);
    }
    let polysyllables = stats.polysyllables.ok_or(FormulaErrorCode::Internal)?;
    let value = polysyllables as f64 * 30.0 / stats.sentences as f64;
    Ok(1.043 * value.sqrt() + 3.1291)
}
