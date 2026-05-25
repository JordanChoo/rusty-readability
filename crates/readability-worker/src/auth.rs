use worker::{Request, Response};

use crate::errors::json_error_response;

pub fn validate_auth(req: &Request, expected: Option<&str>) -> Result<(), Response> {
    let expected = match expected {
        Some(key) if !key.is_empty() => key,
        _ => return Err(json_error_response(500, "INTERNAL_ERROR", "Auth not configured")),
    };

    let url = req.url().map_err(|_| {
        json_error_response(500, "INTERNAL_ERROR", "Failed to parse request URL")
    })?;

    let provided = url
        .query_pairs()
        .find(|(k, _)| k == "csvkey")
        .map(|(_, v)| v.into_owned());

    let provided = match provided {
        Some(key) => key,
        None => return Err(json_error_response(401, "UNAUTHORIZED", "Unauthorized")),
    };

    if !constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        return Err(json_error_response(401, "UNAUTHORIZED", "Unauthorized"));
    }

    Ok(())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
