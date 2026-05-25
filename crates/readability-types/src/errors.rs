use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FormulaErrorCode {
    DivisionByZero,
    Timeout,
    Internal,
    InsufficientText,
}

impl fmt::Display for FormulaErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivisionByZero => write!(f, "Division by zero: input has zero sentences or zero words"),
            Self::Timeout => write!(f, "CPU budget exhausted before formula completed"),
            Self::Internal => write!(f, "Unexpected internal error in formula computation"),
            Self::InsufficientText => write!(f, "Insufficient text for formula computation"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WarningSeverity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for WarningSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WarningCode {
    EmptyText,
    ShortText,
    FewSentences,
    NoSentenceTerminator,
    SmogShortSample,
    SpacheScope,
    NonEnglishUnsupported,
    HtmlMalformed,
    MarkdownCodeIgnored,
    TokenizerAmbiguity,
    SyllableEstimate,
    NormalizedScore,
    DebugOutputCapped,
    TextHashDisabled,
    PartialResult,
    NoValidMetrics,
    FormulaError,
}

impl WarningCode {
    pub fn default_severity(&self) -> WarningSeverity {
        match self {
            Self::EmptyText | Self::NoValidMetrics => WarningSeverity::Error,
            Self::FewSentences | Self::SmogShortSample | Self::SpacheScope
            | Self::NonEnglishUnsupported | Self::PartialResult | Self::FormulaError => {
                WarningSeverity::Warning
            }
            Self::ShortText | Self::NoSentenceTerminator | Self::HtmlMalformed
            | Self::MarkdownCodeIgnored | Self::TokenizerAmbiguity | Self::SyllableEstimate
            | Self::NormalizedScore | Self::DebugOutputCapped | Self::TextHashDisabled => {
                WarningSeverity::Info
            }
        }
    }

    pub fn default_message(&self) -> &'static str {
        match self {
            Self::EmptyText => "No analyzable words in the provided text.",
            Self::ShortText => "Scores are less stable below 100 words.",
            Self::FewSentences => "Fewer than 3 sentences detected.",
            Self::NoSentenceTerminator => "Text has words but no terminal punctuation; fallback sentence construction was used.",
            Self::SmogShortSample => "SMOG is most reliable on 30 or more sentences.",
            Self::SpacheScope => "Spache is intended primarily for early-grade texts.",
            Self::NonEnglishUnsupported => "Non-English text detected or requested; only English is supported in v1.",
            Self::HtmlMalformed => "HTML cleanup was uncertain; results may be affected.",
            Self::MarkdownCodeIgnored => "Fenced code blocks were skipped during analysis.",
            Self::TokenizerAmbiguity => "Abbreviation or acronym segmentation was uncertain.",
            Self::SyllableEstimate => "Syllable counts rely heavily on rule-based estimation (>80%).",
            Self::NormalizedScore => "Clamped normalized display values are included.",
            Self::DebugOutputCapped => "Debug word/token output was capped.",
            Self::TextHashDisabled => "Text hash output is disabled.",
            Self::PartialResult => "CPU budget exceeded; some metrics are incomplete.",
            Self::NoValidMetrics => "All metrics failed; no consensus is possible.",
            Self::FormulaError => "A specific formula produced an error.",
        }
    }
}

impl fmt::Display for WarningCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.default_message())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub code: WarningCode,
    pub message: String,
    pub severity: WarningSeverity,
}

impl Warning {
    pub fn new(code: WarningCode) -> Self {
        let severity = code.default_severity();
        let message = code.default_message().to_string();
        Self {
            code,
            message,
            severity,
        }
    }

    pub fn with_message(code: WarningCode, message: impl Into<String>) -> Self {
        let severity = code.default_severity();
        Self {
            code,
            message: message.into(),
            severity,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnalyzeErrorCode {
    InvalidJson,
    MissingText,
    EmptyText,
    PayloadTooLarge,
    UnsupportedContentType,
    Unauthorized,
    InternalError,
    InternalPanic,
    MethodNotAllowed,
    NotFound,
    BatchTooLarge,
    BatchTooManyTexts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: AnalyzeErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}
