# Worker Integration Tests

End-to-end tests for the Cloudflare Worker HTTP layer.

## Prerequisites

- Node.js >= 22 (use `nvm use 22`)
- `wrangler` CLI
- `worker-build` (`cargo install "worker-build@^0.1"`)

## Running

1. Start wrangler dev in one terminal:

```bash
cd /data/projects/rusty-readability
source ~/.nvm/nvm.sh && nvm use 22
CSVKEY=test-key-integration-12345 npx wrangler dev --port 8787
```

2. Run tests in another terminal:

```bash
./tests/worker-integration/run.sh
```

Or specify a custom port:

```bash
PORT=8788 ./tests/worker-integration/run.sh
```

## Test coverage

- Health endpoint (no auth)
- OpenAPI spec endpoint (no auth)
- Authentication (missing, wrong, valid csvkey)
- POST /v1/analyze (valid, empty, invalid JSON, wrong content-type, oversized)
- Response formats (full, compact, scores_only)
- ETag caching (presence, 304, determinism)
- CORS (preflight, response headers)
- Batch analysis (valid, mixed, too many)
- Metrics and version endpoints
- Method errors (GET on POST-only, PUT)
- 404 handling
- HTML input detection
- No NaN/Infinity in JSON output
