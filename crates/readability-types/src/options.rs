use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputFormat {
    Auto,
    Plain,
    Html,
    MarkdownLite,
}

impl Default for InputFormat {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    Full,
    Compact,
    ScoresOnly,
}

impl Default for ResponseFormat {
    fn default() -> Self {
        Self::Full
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryScoreType {
    ConsensusGrade,
}

impl Default for PrimaryScoreType {
    fn default() -> Self {
        Self::ConsensusGrade
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeOptions {
    #[serde(default)]
    pub input_format: InputFormat,

    #[serde(default = "default_language")]
    pub language: String,

    #[serde(default)]
    pub primary_score: PrimaryScoreType,

    #[serde(default)]
    pub response_format: ResponseFormat,

    #[serde(default = "default_round")]
    pub round: u8,

    #[serde(default)]
    pub normalize: bool,

    #[serde(default = "default_true")]
    pub include_stats: bool,

    #[serde(default = "default_true")]
    pub include_explanations: bool,

    #[serde(default)]
    pub include_paragraphs: bool,

    #[serde(default)]
    pub include_hardest_sentences: u8,

    #[serde(default)]
    pub include_difficult_words: bool,

    #[serde(default)]
    pub include_debug: bool,

    #[serde(default = "default_true")]
    pub hash_text: bool,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            input_format: InputFormat::default(),
            language: default_language(),
            primary_score: PrimaryScoreType::default(),
            response_format: ResponseFormat::default(),
            round: default_round(),
            normalize: false,
            include_stats: true,
            include_explanations: true,
            include_paragraphs: false,
            include_hardest_sentences: 0,
            include_difficult_words: false,
            include_debug: false,
            hash_text: true,
        }
    }
}

fn default_language() -> String {
    "en-US".to_string()
}

fn default_round() -> u8 {
    2
}

fn default_true() -> bool {
    true
}
