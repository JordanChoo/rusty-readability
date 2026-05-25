use sha2::{Digest, Sha256};

use readability_types::{
    AnalysisResult, AnalyzeOptions, Confidence, DaleChallScore, EngineMetadata, HardestSentence,
    InputFormat, InputMetadata, ParagraphBreakdown, PrimaryResult, ResponseStats, Score, Scores,
    WordListVersions,
};

use crate::consensus::{compute_agreement, compute_consensus};
use crate::formulas;
use crate::interpretation::{flesch_ease_interpretation, grade_band};
use crate::paragraphs::{compute_paragraph_breakdown, find_hardest_sentences};
use crate::preprocess;
use crate::stats::compute_stats;
use crate::warnings::generate_warnings;

const MAX_WORDS_BUDGET: usize = 50_000;

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("No analyzable text provided")]
    EmptyText,
    #[error("Internal error: {0}")]
    Internal(String),
}

pub fn analyze(input: &str, options: &AnalyzeOptions) -> Result<AnalysisResult, AnalyzeError> {
    let detected_format = match options.input_format {
        InputFormat::Auto => preprocess::detect::detect_format(input),
        InputFormat::Plain => InputFormat::Plain,
        InputFormat::Html => InputFormat::Html,
        InputFormat::MarkdownLite => InputFormat::MarkdownLite,
    };

    let plain_text = match detected_format {
        InputFormat::Plain | InputFormat::Auto => preprocess::plain::preprocess_plain(input),
        InputFormat::Html => preprocess::html::preprocess_html(input),
        InputFormat::MarkdownLite => preprocess::markdown_lite::preprocess_markdown(input),
    };

    if plain_text.trim().is_empty() {
        return Err(AnalyzeError::EmptyText);
    }

    let result = compute_stats(&plain_text);
    let stats = &result.stats;

    if stats.words == 0 {
        return Err(AnalyzeError::EmptyText);
    }

    let round = options.round;
    let budget_exceeded = stats.words > MAX_WORDS_BUDGET;

    let text_hash = if options.hash_text {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        Some(format!("{:x}", hasher.finalize()))
    } else {
        None
    };

    let mut warnings = generate_warnings(stats);
    if budget_exceeded {
        warnings.push(readability_types::Warning::new(
            readability_types::WarningCode::PartialResult,
        ));
    }

    // Compute all formulas
    let flesch_ease = formulas::flesch::flesch_reading_ease(stats);
    let flesch_kincaid = formulas::flesch_kincaid::flesch_kincaid_grade(stats);
    let fog = formulas::gunning_fog::gunning_fog(stats);
    let cl = formulas::coleman_liau::coleman_liau(stats);
    let smog_result = formulas::smog::smog(stats);
    let ari_result = formulas::ari::automated_readability_index(stats);
    let dc_result = formulas::dale_chall::dale_chall(stats);
    let spache_result = formulas::spache::spache(stats);

    // Build scores
    let scores = Scores {
        flesch_reading_ease: Some(build_flesch_ease_score(&flesch_ease, round)),
        flesch_kincaid_grade: Some(build_grade_score(&flesch_kincaid, "flesch_kincaid", round)),
        gunning_fog: Some(build_grade_score(&fog, "gunning_fog", round)),
        coleman_liau: Some(build_grade_score(&cl, "coleman_liau", round)),
        smog: Some(build_grade_score(&smog_result, "smog", round)),
        automated_readability_index: Some(build_grade_score(&ari_result, "ari", round)),
        dale_chall: Some(build_dale_chall_score(&dc_result, round)),
        spache: Some(build_grade_score(&spache_result, "spache", round)),
    };

    // Collect grade estimates for consensus
    let mut grade_estimates: Vec<f64> = Vec::new();
    let mut included_metrics: Vec<String> = Vec::new();

    if let Ok(v) = &flesch_kincaid {
        grade_estimates.push(*v);
        included_metrics.push("flesch_kincaid_grade".to_string());
    }
    if let Ok(v) = &fog {
        grade_estimates.push(*v);
        included_metrics.push("gunning_fog".to_string());
    }
    if let Ok(v) = &cl {
        grade_estimates.push(*v);
        included_metrics.push("coleman_liau".to_string());
    }
    if let Ok(v) = &smog_result {
        grade_estimates.push(*v);
        included_metrics.push("smog".to_string());
    }
    if let Ok(v) = &ari_result {
        grade_estimates.push(*v);
        included_metrics.push("ari".to_string());
    }
    if let Ok(dc) = &dc_result {
        grade_estimates.push(formulas::dale_chall::dale_chall_grade_band(dc.adjusted).1);
        included_metrics.push("dale_chall".to_string());
    }
    if let Ok(v) = &spache_result {
        grade_estimates.push(*v);
        included_metrics.push("spache".to_string());
    }

    let consensus = compute_consensus(&grade_estimates);
    let agreement = compute_agreement(&grade_estimates);

    let confidence = match grade_estimates.len() {
        0 => Confidence::Low,
        1..=3 => Confidence::Medium,
        _ => match &agreement {
            Some(a) if a.spread <= 3.0 => Confidence::High,
            Some(a) if a.spread <= 6.0 => Confidence::Medium,
            _ => Confidence::Low,
        },
    };

    let primary = PrimaryResult {
        score_type: "consensus_grade".to_string(),
        score: consensus.map(|v| round_to(v, round)),
        label: consensus.map(|v| grade_band(v)),
        ease_score: flesch_ease.as_ref().ok().map(|v| round_to(*v, round)),
        ease_label: flesch_ease
            .as_ref()
            .ok()
            .map(|v| flesch_ease_interpretation(*v).to_string()),
        confidence,
        agreement,
        included_metrics: Some(included_metrics),
        excluded_metrics: None,
        drivers: None,
        summary: consensus.map(|v| {
            format!(
                "This text is at a {} reading level.",
                grade_band(v)
            )
        }),
    };

    // Build response stats
    let response_stats = if options.include_stats {
        Some(ResponseStats {
            sentences: stats.sentences,
            words: stats.words,
            unique_words: stats.unique_words,
            letters: stats.letters_ascii,
            ari_characters: stats.ari_characters,
            graphemes: stats.graphemes,
            paragraphs: stats.paragraphs,
            syllables: stats.syllables,
            polysyllables: stats.polysyllables,
            complex_words: stats.complex_words,
            dale_chall_difficult_words: stats.dale_chall_difficult_words,
            dale_chall_difficult_percentage: stats.dale_chall_difficult_percentage.map(|v| round_to(v, round)),
            spache_unique_unfamiliar_words: stats.spache_unique_unfamiliar_words,
            spache_unique_unfamiliar_percentage: stats.spache_unique_unfamiliar_percentage.map(|v| round_to(v, round)),
            average_words_per_sentence: round_to(stats.average_words_per_sentence, round),
            average_syllables_per_word: stats.average_syllables_per_word.map(|v| round_to(v, round)),
            average_characters_per_word: round_to(stats.average_characters_per_word, round),
            longest_sentence_words: stats.longest_sentence_words,
            type_token_ratio: round_to(
                if stats.words > 0 { stats.unique_words as f64 / stats.words as f64 } else { 0.0 },
                round,
            ),
            hapax_percentage: round_to(
                if stats.words > 0 { 100.0 * stats.hapax_count as f64 / stats.words as f64 } else { 0.0 },
                round,
            ),
            reading_time_seconds: (stats.words as f64 / 238.0 * 60.0).round() as u64,
            reading_time_wpm: 238,
        })
    } else {
        None
    };

    let paragraphs: Option<Vec<ParagraphBreakdown>> = if options.include_paragraphs {
        Some(compute_paragraph_breakdown(
            &result.sentence_infos,
            stats.paragraphs,
        ))
    } else {
        None
    };

    let hardest_sentences: Option<Vec<HardestSentence>> = if options.include_hardest_sentences > 0 {
        Some(find_hardest_sentences(
            &result.sentence_infos,
            options.include_hardest_sentences as usize,
        ))
    } else {
        None
    };

    let format_str = match detected_format {
        InputFormat::Auto | InputFormat::Plain => "plain",
        InputFormat::Html => "html",
        InputFormat::MarkdownLite => "markdown_lite",
    };

    let request_id = format!("{:x}", Sha256::digest(input.as_bytes()))[..16].to_string();

    Ok(AnalysisResult {
        schema_version: "1.0".to_string(),
        request_id,
        engine: Some(EngineMetadata {
            formula_version: "1.0.0".to_string(),
            tokenizer_version: "1.0.0".to_string(),
            syllable_version: "1.0.0".to_string(),
            character_count_policy: "ascii_letters".to_string(),
            word_lists: WordListVersions {
                dale_chall: "2024-public-domain".to_string(),
                spache: "2024-public-domain".to_string(),
            },
        }),
        input: Some(InputMetadata {
            language: options.language.clone(),
            input_format_detected: format_str.to_string(),
            bytes: input.len(),
            text_hash,
            warnings: Vec::new(),
        }),
        stats: response_stats,
        scores,
        primary,
        paragraphs,
        hardest_sentences,
        warnings: if warnings.is_empty() {
            None
        } else {
            Some(warnings)
        },
    })
}

pub fn analyze_batch(
    inputs: &[(&str, &str)],
    options: &AnalyzeOptions,
) -> Vec<(String, Result<AnalysisResult, AnalyzeError>)> {
    inputs
        .iter()
        .map(|(id, text)| (id.to_string(), analyze(text, options)))
        .collect()
}

fn build_flesch_ease_score(
    result: &Result<f64, readability_types::FormulaErrorCode>,
    round: u8,
) -> Score {
    match result {
        Ok(raw) => {
            let score = round_to(*raw, round);
            Score {
                raw: *raw,
                score,
                normalized_0_100: Some(raw.clamp(0.0, 100.0)),
                higher_is_easier: Some(true),
                interpretation: Some(flesch_ease_interpretation(*raw).to_string()),
                grade_band: None,
                valid: true,
                confidence: if *raw >= 0.0 && *raw <= 100.0 {
                    Confidence::High
                } else {
                    Confidence::Medium
                },
                error: None,
                warnings: Vec::new(),
            }
        }
        Err(e) => Score {
            raw: 0.0,
            score: 0.0,
            normalized_0_100: None,
            higher_is_easier: Some(true),
            interpretation: None,
            grade_band: None,
            valid: false,
            confidence: Confidence::Low,
            error: Some(e.clone()),
            warnings: Vec::new(),
        },
    }
}

fn build_grade_score(
    result: &Result<f64, readability_types::FormulaErrorCode>,
    _name: &str,
    round: u8,
) -> Score {
    match result {
        Ok(raw) => {
            let score = round_to(*raw, round);
            Score {
                raw: *raw,
                score,
                normalized_0_100: None,
                higher_is_easier: Some(false),
                interpretation: None,
                grade_band: Some(grade_band(*raw)),
                valid: true,
                confidence: Confidence::High,
                error: None,
                warnings: Vec::new(),
            }
        }
        Err(e) => Score {
            raw: 0.0,
            score: 0.0,
            normalized_0_100: None,
            higher_is_easier: Some(false),
            interpretation: None,
            grade_band: None,
            valid: false,
            confidence: Confidence::Low,
            error: Some(e.clone()),
            warnings: Vec::new(),
        },
    }
}

fn build_dale_chall_score(
    result: &Result<formulas::dale_chall::DaleChallResult, readability_types::FormulaErrorCode>,
    round: u8,
) -> DaleChallScore {
    match result {
        Ok(dc) => {
            let (band, midpoint) = formulas::dale_chall::dale_chall_grade_band(dc.adjusted);
            DaleChallScore {
                raw_unadjusted: dc.raw_unadjusted,
                raw: dc.adjusted,
                score: round_to(dc.adjusted, round),
                grade_band: Some(band.to_string()),
                grade_midpoint: Some(midpoint),
                valid: true,
                confidence: Confidence::High,
                error: None,
                warnings: Vec::new(),
            }
        }
        Err(e) => DaleChallScore {
            raw_unadjusted: 0.0,
            raw: 0.0,
            score: 0.0,
            grade_band: None,
            grade_midpoint: None,
            valid: false,
            confidence: Confidence::Low,
            error: Some(e.clone()),
            warnings: Vec::new(),
        },
    }
}

fn round_to(value: f64, decimals: u8) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_opts() -> AnalyzeOptions {
        AnalyzeOptions::default()
    }

    #[test]
    fn analyze_simple_text() {
        let result = analyze("The cat sat on the mat. The dog ran fast.", &default_opts()).unwrap();
        assert_eq!(result.schema_version, "1.0");
        assert!(result.stats.is_some());
        let stats = result.stats.unwrap();
        assert_eq!(stats.sentences, 2);
        assert!(stats.words > 0);
        assert!(result.scores.flesch_reading_ease.is_some());
        assert!(result.primary.score.is_some());
    }

    #[test]
    fn analyze_empty_text() {
        let result = analyze("", &default_opts());
        assert!(result.is_err());
    }

    #[test]
    fn analyze_whitespace_only() {
        let result = analyze("   \n\n  \t  ", &default_opts());
        assert!(result.is_err());
    }

    #[test]
    fn analyze_with_paragraphs() {
        let mut opts = default_opts();
        opts.include_paragraphs = true;
        let result = analyze("First paragraph.\n\nSecond paragraph.", &opts).unwrap();
        assert!(result.paragraphs.is_some());
    }

    #[test]
    fn analyze_with_hardest_sentences() {
        let mut opts = default_opts();
        opts.include_hardest_sentences = 2;
        let text = "The cat sat. The incredibly sophisticated international conglomerate established unprecedented organizational frameworks.";
        let result = analyze(text, &opts).unwrap();
        assert!(result.hardest_sentences.is_some());
        assert!(!result.hardest_sentences.unwrap().is_empty());
    }

    #[test]
    fn analyze_produces_text_hash() {
        let result = analyze("Hello world.", &default_opts()).unwrap();
        assert!(result.input.unwrap().text_hash.is_some());
    }

    #[test]
    fn analyze_no_hash_when_disabled() {
        let mut opts = default_opts();
        opts.hash_text = false;
        let result = analyze("Hello world.", &opts).unwrap();
        assert!(result.input.unwrap().text_hash.is_none());
    }

    #[test]
    fn analyze_all_scores_valid() {
        let text = "The quick brown fox jumped over the lazy dog. Simple sentences are easy to read. Complex vocabulary increases difficulty.";
        let result = analyze(text, &default_opts()).unwrap();
        assert!(result.scores.flesch_reading_ease.unwrap().valid);
        assert!(result.scores.flesch_kincaid_grade.unwrap().valid);
        assert!(result.scores.gunning_fog.unwrap().valid);
        assert!(result.scores.coleman_liau.unwrap().valid);
        assert!(result.scores.smog.unwrap().valid);
        assert!(result.scores.automated_readability_index.unwrap().valid);
        assert!(result.scores.dale_chall.unwrap().valid);
        assert!(result.scores.spache.unwrap().valid);
    }

    #[test]
    fn analyze_batch_works() {
        let inputs = vec![
            ("doc1", "Hello world. Simple text."),
            ("doc2", "The cat sat on the mat."),
        ];
        let results = analyze_batch(&inputs, &default_opts());
        assert_eq!(results.len(), 2);
        assert!(results[0].1.is_ok());
        assert!(results[1].1.is_ok());
    }

    #[test]
    fn round_to_works() {
        assert_eq!(round_to(3.14159, 2), 3.14);
        assert_eq!(round_to(3.14159, 0), 3.0);
        assert_eq!(round_to(3.14159, 4), 3.1416);
    }
}
