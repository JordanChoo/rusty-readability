use worker::{Headers, Response, Result};

const ALLOWED_ORIGINS: &str = "*";
const ALLOWED_METHODS: &str = "GET, POST, OPTIONS";
const ALLOWED_HEADERS: &str = "Content-Type";
const MAX_AGE: &str = "86400";

pub fn preflight_response() -> Result<Response> {
    let headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", ALLOWED_ORIGINS)?;
    headers.set("Access-Control-Allow-Methods", ALLOWED_METHODS)?;
    headers.set("Access-Control-Allow-Headers", ALLOWED_HEADERS)?;
    headers.set("Access-Control-Max-Age", MAX_AGE)?;
    Ok(Response::empty()?.with_status(204).with_headers(headers))
}

pub fn with_cors(response: Response) -> Result<Response> {
    let headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", ALLOWED_ORIGINS)?;

    let existing = response.headers().clone();
    for (key, value) in existing.entries() {
        headers.set(&key, &value)?;
    }

    Ok(response.with_headers(headers))
}
