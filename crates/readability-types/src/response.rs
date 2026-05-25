use serde::{Deserialize, Serialize};

use crate::errors::{FormulaErrorCode, Warning};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub schema_version: String,
    pub request_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<EngineMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<ResponseStats>,

    pub scores: Scores,
    pub primary: PrimaryResult,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraphs: Option<Vec<ParagraphBreakdown>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardest_sentences: Option<Vec<HardestSentence>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<Warning>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetadata {
    pub formula_version: String,
    pub tokenizer_version: String,
    pub syllable_version: String,
    pub character_count_policy: String,
    pub word_lists: WordListVersions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordListVersions {
    pub dale_chall: String,
    pub spache: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMetadata {
    pub language: String,
    pub input_format_detected: String,
    pub bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_hash: Option<String>,
    #[serde(default)]
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStats {
    pub sentences: usize,
    pub words: usize,
    pub unique_words: usize,
    pub letters: usize,
    pub ari_characters: usize,
    pub graphemes: usize,
    pub paragraphs: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub syllables: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polysyllables: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complex_words: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dale_chall_difficult_words: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dale_chall_difficult_percentage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spache_unique_unfamiliar_words: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spache_unique_unfamiliar_percentage: Option<f64>,

    pub average_words_per_sentence: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_syllables_per_word: Option<f64>,
    pub average_characters_per_word: f64,
    pub longest_sentence_words: usize,

    pub type_token_ratio: f64,
    pub hapax_percentage: f64,
    pub reading_time_seconds: u64,
    pub reading_time_wpm: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scores {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flesch_reading_ease: Option<Score>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flesch_kincaid_grade: Option<Score>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gunning_fog: Option<Score>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coleman_liau: Option<Score>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smog: Option<Score>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automated_readability_index: Option<Score>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dale_chall: Option<DaleChallScore>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spache: Option<Score>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub raw: f64,
    pub score: f64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_0_100: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub higher_is_easier: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade_band: Option<String>,

    pub valid: bool,
    pub confidence: Confidence,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<FormulaErrorCode>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaleChallScore {
    pub raw_unadjusted: f64,
    pub raw: f64,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade_band: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade_midpoint: Option<f64>,
    pub valid: bool,
    pub confidence: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<FormulaErrorCode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Warning>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Stability {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryResult {
    #[serde(rename = "type")]
    pub score_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ease_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ease_label: Option<String>,
    pub confidence: Confidence,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agreement: Option<Agreement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_metrics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_metrics: Option<Vec<ExcludedMetric>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drivers: Option<Vec<Driver>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agreement {
    pub min_grade: f64,
    pub max_grade: f64,
    pub spread: f64,
    pub stability: Stability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludedMetric {
    pub metric: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    pub factor: String,
    pub direction: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphBreakdown {
    pub index: usize,
    pub words: usize,
    pub sentences: usize,
    pub avg_sentence_length: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_syllables_per_word: Option<f64>,
    pub difficulty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardestSentence {
    pub index: usize,
    pub paragraph_index: usize,
    pub words: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syllables_per_word: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complex_word_count: Option<usize>,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    pub schema_version: String,
    pub request_id: String,
    pub results: Vec<BatchResultItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResultItem {
    pub id: String,
    pub status: BatchItemStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<AnalysisResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BatchItemError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchItemStatus {
    Ok,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoresOnlyResponse {
    pub primary: ScoresOnlyPrimary,
    pub scores: ScoresOnlyScores,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoresOnlyPrimary {
    pub score: Option<f64>,
    pub label: Option<String>,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoresOnlyScores {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flesch_reading_ease: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flesch_kincaid_grade: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gunning_fog: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coleman_liau: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smog: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ari: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dale_chall: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spache: Option<f64>,
}
