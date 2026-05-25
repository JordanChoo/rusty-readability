use sha2::{Digest, Sha256};
use worker::{Request, Response, Result};

use readability_core::analyze::{analyze, analyze_batch};
use readability_types::{
    AnalysisResult, AnalyzeRequest, BatchRequest, ResponseFormat, ScoresOnlyPrimary,
    ScoresOnlyResponse, ScoresOnlyScores,
};

use crate::errors::json_error_response;

const OPENAPI_SPEC: &str = include_str!("../../../openapi.json");

const MAX_BODY_BYTES: usize = 256 * 1024;
const MAX_BATCH_BODY_BYTES: usize = 1024 * 1024;
const MAX_BATCH_TEXTS: usize = 10;

pub fn health() -> Result<Response> {
    Response::from_json(&serde_json::json!({
        "status": "ok",
        "engine": "rusty-readability",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub fn metrics() -> Result<Response> {
    Response::from_json(&serde_json::json!({
        "metrics": [
            "flesch_reading_ease",
            "flesch_kincaid_grade",
            "gunning_fog",
            "coleman_liau",
            "smog",
            "automated_readability_index",
            "dale_chall",
            "spache"
        ],
        "primary_score_types": ["consensus_grade"],
        "input_formats": ["auto", "plain", "html", "markdown_lite"],
        "response_formats": ["full", "compact", "scores_only"]
    }))
}

pub fn openapi_spec() -> Result<Response> {
    let headers = worker::Headers::new();
    headers.set("Content-Type", "application/json")?;
    Ok(Response::ok(OPENAPI_SPEC)?.with_headers(headers))
}

pub fn version() -> Result<Response> {
    Response::from_json(&serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "engine": {
            "formula_version": "1.0.0",
            "tokenizer_version": "1.0.0",
            "syllable_version": "1.0.0",
            "word_lists": {
                "dale_chall": "2024-public-domain",
                "spache": "2024-public-domain"
            }
        }
    }))
}

pub async fn analyze_endpoint(mut req: Request) -> Result<Response> {
    let content_type = req.headers().get("Content-Type")?.unwrap_or_default();

    if !content_type.contains("application/json") {
        return Ok(json_error_response(
            415,
            "UNSUPPORTED_CONTENT_TYPE",
            "Content-Type must be application/json",
        ));
    }

    let if_none_match = req.headers().get("If-None-Match")?.unwrap_or_default();

    let body = req.text().await?;

    if body.len() > MAX_BODY_BYTES {
        return Ok(json_error_response(413, "PAYLOAD_TOO_LARGE", "Payload too large"));
    }

    let request: AnalyzeRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("Invalid JSON: {e}");
            return Ok(json_error_response(400, "INVALID_JSON", &msg));
        }
    };

    if request.text.is_empty() {
        return Ok(json_error_response(
            400,
            "MISSING_TEXT",
            "text field is required and must not be empty",
        ));
    }

    let etag = compute_etag(&request.text, &request.options);

    if !if_none_match.is_empty() && if_none_match.trim_matches('"') == etag.trim_matches('"') {
        let mut resp = Response::empty()?.with_status(304);
        resp.headers_mut().set("ETag", &etag)?;
        return Ok(resp);
    }

    match analyze(&request.text, &request.options) {
        Ok(result) => {
            let mut resp = format_response(&result, &request.options.response_format)?;
            resp.headers_mut().set("ETag", &etag)?;
            Ok(resp)
        }
        Err(e) => Ok(json_error_response(422, "EMPTY_TEXT", &e.to_string())),
    }
}

pub async fn batch(mut req: Request) -> Result<Response> {
    let content_type = req.headers().get("Content-Type")?.unwrap_or_default();

    if !content_type.contains("application/json") {
        return Ok(json_error_response(
            415,
            "UNSUPPORTED_CONTENT_TYPE",
            "Content-Type must be application/json",
        ));
    }

    let body = req.text().await?;

    if body.len() > MAX_BATCH_BODY_BYTES {
        return Ok(json_error_response(
            413,
            "PAYLOAD_TOO_LARGE",
            "Batch payload too large",
        ));
    }

    let request: BatchRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("Invalid JSON: {e}");
            return Ok(json_error_response(400, "INVALID_JSON", &msg));
        }
    };

    if request.texts.len() > MAX_BATCH_TEXTS {
        return Ok(json_error_response(
            400,
            "BATCH_TOO_MANY_TEXTS",
            &format!("Maximum {MAX_BATCH_TEXTS} texts per batch request"),
        ));
    }

    let inputs: Vec<(&str, &str)> = request
        .texts
        .iter()
        .map(|t| (t.id.as_str(), t.text.as_str()))
        .collect();

    let results = analyze_batch(&inputs, &request.options);

    let batch_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(id, result)| match result {
            Ok(analysis) => serde_json::json!({
                "id": id,
                "status": "ok",
                "data": analysis
            }),
            Err(e) => serde_json::json!({
                "id": id,
                "status": "error",
                "error": {
                    "code": "ANALYSIS_ERROR",
                    "message": e.to_string()
                }
            }),
        })
        .collect();

    Response::from_json(&serde_json::json!({
        "schema_version": "1.0",
        "results": batch_results
    }))
}

fn format_response(result: &AnalysisResult, format: &ResponseFormat) -> Result<Response> {
    match format {
        ResponseFormat::Full => Response::from_json(result),
        ResponseFormat::Compact => {
            let compact = serde_json::json!({
                "schema_version": result.schema_version,
                "request_id": result.request_id,
                "scores": result.scores,
                "primary": result.primary,
                "stats": result.stats,
                "warnings": result.warnings,
            });
            Response::from_json(&compact)
        }
        ResponseFormat::ScoresOnly => {
            let scores_only = ScoresOnlyResponse {
                primary: ScoresOnlyPrimary {
                    score: result.primary.score,
                    label: result.primary.label.clone(),
                    confidence: result.primary.confidence,
                },
                scores: ScoresOnlyScores {
                    flesch_reading_ease: result
                        .scores
                        .flesch_reading_ease
                        .as_ref()
                        .filter(|s| s.valid)
                        .map(|s| s.score),
                    flesch_kincaid_grade: result
                        .scores
                        .flesch_kincaid_grade
                        .as_ref()
                        .filter(|s| s.valid)
                        .map(|s| s.score),
                    gunning_fog: result
                        .scores
                        .gunning_fog
                        .as_ref()
                        .filter(|s| s.valid)
                        .map(|s| s.score),
                    coleman_liau: result
                        .scores
                        .coleman_liau
                        .as_ref()
                        .filter(|s| s.valid)
                        .map(|s| s.score),
                    smog: result
                        .scores
                        .smog
                        .as_ref()
                        .filter(|s| s.valid)
                        .map(|s| s.score),
                    ari: result
                        .scores
                        .automated_readability_index
                        .as_ref()
                        .filter(|s| s.valid)
                        .map(|s| s.score),
                    dale_chall: result
                        .scores
                        .dale_chall
                        .as_ref()
                        .filter(|s| s.valid)
                        .map(|s| s.score),
                    spache: result
                        .scores
                        .spache
                        .as_ref()
                        .filter(|s| s.valid)
                        .map(|s| s.score),
                },
                text_hash: result.input.as_ref().and_then(|i| i.text_hash.clone()),
            };
            Response::from_json(&scores_only)
        }
    }
}

fn compute_etag(text: &str, options: &readability_types::AnalyzeOptions) -> String {
    let text_hash = format!("{:x}", Sha256::digest(text.as_bytes()));
    let opts_json = serde_json::to_string(options).unwrap_or_default();
    let opts_hash = format!("{:x}", Sha256::digest(opts_json.as_bytes()));
    format!("\"v1-{}-{}\"", &text_hash[..16], &opts_hash[..16])
}
