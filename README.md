# rusty-readability

A production-grade readability analysis API written in Rust and deployed to Cloudflare Workers. Implements 8 canonical readability formulas from their published academic definitions, returns a practical consensus grade, raw statistics, per-paragraph breakdowns, and actionable warnings. Privacy-preserving by default: submitted text is never stored or logged.

## Quick Start

```bash
curl -X POST "https://your-worker.workers.dev/v1/analyze?csvkey=YOUR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"text": "The cat sat on the mat. The dog ran fast."}'
```

Response (abbreviated):

```json
{
  "primary": {
    "type": "consensus_grade",
    "score": 1.82,
    "label": "2nd grade",
    "ease_score": 107.73,
    "ease_label": "very easy",
    "confidence": "low",
    "summary": "This text is at a 2nd grade reading level."
  },
  "scores": {
    "flesch_reading_ease": { "score": 107.73, "interpretation": "very easy" },
    "flesch_kincaid_grade": { "score": 0.26, "grade_band": "below 1st grade" },
    "gunning_fog": { "score": 3.6, "grade_band": "4th grade" },
    "...": "..."
  },
  "warnings": [
    { "code": "SHORT_TEXT", "message": "Scores are less stable below 100 words." }
  ]
}
```

## Understanding the Scores

The `primary.score` is a consensus grade level derived from all valid formula outputs. A score of 8.9 means "approximately 9th grade reading level."

| Score Range | Reading Level |
|-------------|---------------|
| < 1 | Below 1st grade |
| 1-6 | Elementary school |
| 7-8 | Middle school |
| 9-12 | High school |
| 13-16 | College |
| > 16 | College graduate / professional |

`primary.ease_score` is the Flesch Reading Ease score (0-100 scale, higher = easier):

| FRE Score | Interpretation |
|-----------|----------------|
| 90-100 | Very easy |
| 80-89 | Easy |
| 70-79 | Fairly easy |
| 60-69 | Standard |
| 50-59 | Fairly difficult |
| 30-49 | Difficult |
| 0-29 | Very confusing |

## API Endpoints

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| `POST` | `/v1/analyze` | Yes | Analyze a single text |
| `POST` | `/v1/batch` | Yes | Analyze up to 10 texts |
| `GET` | `/v1/metrics` | Yes | List supported metrics and formats |
| `GET` | `/v1/version` | Yes | Engine and word-list versions |
| `GET` | `/v1/openapi.json` | No | OpenAPI 3.0.3 specification |
| `GET` | `/health` | No | Health check |
| `OPTIONS` | `/*` | No | CORS preflight |

Authentication: append `?csvkey=YOUR_KEY` to all authenticated endpoints.

## Request Format

```json
{
  "text": "Your text to analyze.",
  "options": {
    "input_format": "auto",
    "response_format": "full",
    "round": 2,
    "include_stats": true,
    "include_paragraphs": false,
    "include_hardest_sentences": 0,
    "hash_text": true
  }
}
```

| Option | Default | Description |
|--------|---------|-------------|
| `input_format` | `auto` | `auto`, `plain`, `html`, `markdown_lite` |
| `response_format` | `full` | `full`, `compact`, `scores_only` |
| `round` | `2` | Decimal places for display rounding |
| `include_stats` | `true` | Include raw text statistics |
| `include_paragraphs` | `false` | Per-paragraph difficulty breakdown |
| `include_hardest_sentences` | `0` | Number of hardest sentences to return |
| `hash_text` | `true` | Include SHA-256 hash for client correlation |
| `include_explanations` | `true` | Include human-readable interpretation labels |
| `include_difficult_words` | `false` | Include Dale-Chall/Spache difficult word details |
| `include_debug` | `false` | Include internal diagnostic fields |
| `normalize` | `false` | Normalize scores to a common 0-100 scale |
| `language` | `en-US` | Language hint (English only in v1) |

## Response Formats

### `full` (default)

Complete response with engine metadata, stats, all scores, primary result, and warnings.

### `compact`

Scores, primary result, stats, and warnings. No engine metadata.

### `scores_only`

Minimal payload for high-volume integrations:

```json
{
  "primary": { "score": 8.9, "label": "9th grade", "confidence": "medium" },
  "scores": {
    "flesch_reading_ease": 62.37,
    "flesch_kincaid_grade": 8.21,
    "gunning_fog": 12.44,
    "coleman_liau": 9.17,
    "smog": 10.39,
    "ari": 8.91,
    "dale_chall": 7.82,
    "spache": 4.98
  },
  "text_hash": "sha256:..."
}
```

## Batch Analysis

Analyze up to 10 texts in a single request:

```bash
curl -X POST "https://your-worker.workers.dev/v1/batch?csvkey=YOUR_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "texts": [
      { "id": "page-1", "text": "First text to analyze." },
      { "id": "page-2", "text": "Second text to analyze." }
    ],
    "options": { "response_format": "compact" }
  }'
```

Each text is analyzed independently. If one text fails (e.g., empty), its result contains an error while others succeed normally.

## ETag Caching

The API is deterministic: same text + same options = same response. Every successful response includes an `ETag` header. Send `If-None-Match` on subsequent requests to get `304 Not Modified` without re-running analysis.

## Supported Formulas

| Formula | Type | Notes |
|---------|------|-------|
| Flesch Reading Ease | Ease score (0-100) | Higher = easier. Developed by Flesch (1948) |
| Flesch-Kincaid Grade | Grade level | Kincaid et al. (1975) |
| Gunning Fog Index | Grade level | Gunning (1952) |
| Coleman-Liau Index | Grade level | Character-based, no syllable counting. Coleman & Liau (1975) |
| SMOG Index | Grade level | Most reliable with 30+ sentences. McLaughlin (1969) |
| Automated Readability Index | Grade level | Character-based. Smith & Senter (1967) |
| Dale-Chall | Adjusted score | Word-list based. Returns both raw and adjusted scores. Dale & Chall (1948/1995) |
| Spache | Grade level | Intended for early-grade texts. Spache (1953/revised) |

Additional metrics included in `stats`: type-token ratio, hapax percentage, reading time (238 WPM per Brysbaert 2019).

## Warning Codes

| Code | Meaning |
|------|---------|
| `SHORT_TEXT` | Fewer than 100 words; scores are less stable |
| `FEW_SENTENCES` | Fewer than 3 sentences detected |
| `NO_SENTENCE_TERMINATOR` | No terminal punctuation found; fallback sentence construction used |
| `SMOG_SHORT_SAMPLE` | Fewer than 30 sentences; SMOG is less reliable |
| `PARTIAL_RESULT` | CPU budget exceeded; some metrics may be incomplete |

## Local Development

### Prerequisites

- Rust (stable toolchain)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Wrangler CLI (for Worker deployment): `npm install -g wrangler`

### Build and Test

```bash
# Run all tests (excludes Worker crate which requires wasm32)
cargo test --workspace --exclude readability-worker

# Check Worker compiles for WASM
cargo check -p readability-worker --target wasm32-unknown-unknown

# Run the CLI
echo "Your text here." | cargo run -p readability-cli -- analyze -
cargo run -p readability-cli -- analyze path/to/file.txt
cargo run -p readability-cli -- version
```

### Snapshot Tests

API response snapshots are checked into git under `crates/readability-core/tests/snapshots/`. When code changes affect output:

```bash
cargo insta test        # Run and see diffs
cargo insta review      # Interactively approve/reject changes
```

## Deployment

### First-time Setup

```bash
# Authenticate with Cloudflare
wrangler login

# Set the authentication secret
wrangler secret put CSVKEY
# Enter your secret key when prompted

# Deploy
wrangler deploy
```

### Configuration

Request limits are compile-time constants in `crates/readability-worker/src/routes.rs`:

| Limit | Value | Description |
|-------|-------|-------------|
| `MAX_BODY_BYTES` | 262,144 (256 KB) | Max single request body |
| `MAX_BATCH_BODY_BYTES` | 1,048,576 (1 MB) | Max batch request body |
| `MAX_BATCH_TEXTS` | 10 | Max texts per batch |

The version reported by `/v1/version` is the crate version from `Cargo.toml` (`env!("CARGO_PKG_VERSION")`).

### Rollback

Cloudflare Workers supports instant rollback to previous deployments via the dashboard or `wrangler rollback`.

## Architecture

```
readability-types   (shared request/response types, no deps)
       ^
readability-core    (pure Rust engine: formulas, tokenizer, syllables, word lists)
       ^
readability-worker  (Cloudflare Worker HTTP wrapper)
readability-cli     (developer CLI tool)
```

The core engine has no Cloudflare-specific dependencies and can be used as a standalone Rust library.

## Privacy

- Submitted text is **never** stored or logged
- Error messages never contain user text
- Allowed logs: request ID, status, duration, byte count, word count, warning codes
- Text hash (SHA-256) is returned for client correlation but not stored server-side
- The `csvkey` secret is never logged

## Known Limitations

- English only (v1)
- Syllable counting uses rule-based estimation with an exception dictionary; accuracy is approximately 85-90% on common English words
- Readability formulas were designed for prose; results on code-heavy documents, poetry, or tabular data may be unreliable
- Short texts (< 100 words) produce less stable scores
- SMOG is most reliable with 30+ sentences
- Spache is intended for early-grade texts; results on advanced texts are informational
- Dale-Chall and Spache depend on fixed word lists that may not include modern vocabulary

## Disclaimer

Readability formulas are estimates. They do not measure factual accuracy, tone, audience fit, inclusivity, persuasion, legal sufficiency, medical safety, or comprehension by a specific reader.

## License

MIT
