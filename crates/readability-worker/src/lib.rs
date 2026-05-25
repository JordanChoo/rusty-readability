use worker::*;

mod auth;
mod cors;
mod errors;
mod routes;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    if req.method() == Method::Options {
        return cors::preflight_response();
    }

    let path = req.path();
    let method = req.method();

    if path == "/health" && method == Method::Get {
        return routes::health();
    }

    if path == "/v1/openapi.json" && method == Method::Get {
        return routes::openapi_spec();
    }

    let csvkey_secret = env
        .secret("CSVKEY")
        .map(|s| s.to_string())
        .ok()
        .or_else(|| env.var("CSVKEY").map(|v| v.to_string()).ok());
    if let Err(resp) = auth::validate_auth(&req, csvkey_secret.as_deref()) {
        return cors::with_cors(resp);
    }

    let response = match (method, path.as_str()) {
        (Method::Post, "/v1/analyze") => routes::analyze_endpoint(req).await,
        (Method::Post, "/v1/batch") => routes::batch(req).await,
        (Method::Get, "/v1/metrics") => routes::metrics(),
        (Method::Get, "/v1/version") => routes::version(),
        (Method::Get, _) | (Method::Post, _) => {
            Ok(errors::json_error_response(404, "NOT_FOUND", "Not found"))
        }
        _ => Ok(errors::json_error_response(405, "METHOD_NOT_ALLOWED", "Method not allowed")),
    };

    let resp = response?;
    cors::with_cors(resp)
}
