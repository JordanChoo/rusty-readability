use serde::{Deserialize, Serialize};

use crate::options::AnalyzeOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    pub text: String,

    #[serde(default = "default_metrics")]
    pub metrics: Vec<String>,

    #[serde(default)]
    pub options: AnalyzeOptions,
}

fn default_metrics() -> Vec<String> {
    vec!["all".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTextItem {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    pub texts: Vec<BatchTextItem>,

    #[serde(default)]
    pub options: AnalyzeOptions,
}
