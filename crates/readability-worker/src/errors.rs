use worker::Response;

pub fn json_error_response(status: u16, code: &str, message: &str) -> Response {
    let body = serde_json::json!({
        "error": {
            "code": code,
            "message": message
        }
    });

    Response::from_json(&body)
        .unwrap_or_else(|_| Response::error("Internal Server Error", 500).unwrap())
        .with_status(status)
}
