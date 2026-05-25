# rusty-readability

A production-grade readability analysis API written in Rust and deployed to Cloudflare Workers. Implements 8 canonical readability formulas from their published academic definitions, returns a practical consensus grade, raw statistics, per-paragraph breakdowns, and actionable warnings. Privacy-preserving by default: submitted text is never stored or logged.

## Why This Exists

Most readability tools return a single number from a single formula. That number varies significantly depending on which formula you pick, how syllables are counted, and which word list version is used. Two tools analyzing the same text routinely disagree by 2-4 grade levels.

rusty-readability runs all 8 major formulas, shows you where they agree and disagree, and synthesizes a consensus grade that's more stable than any single formula. The response also shows which sentences are hardest, which paragraphs are most complex, and what to watch out for (short text, too few sentences, vocabulary outside the word lists).

It's useful for:

- **Content teams** checking whether copy meets a target reading level
- **SEO pipelines** scoring content readability at scale (batch API, ETag caching)
- **Accessibility audits** verifying plain-language compliance
- **Education platforms** matching content to student grade levels
- **Editorial tools** identifying specific sentences or paragraphs to simplify

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

## How the Consensus Grade Works

Individual readability formulas often disagree by several grade levels on the same text. The `primary.score` is a **consensus grade** that synthesizes all grade-level formulas into a single, more stable estimate. It excludes Flesch Reading Ease, which uses a 0-100 ease scale rather than a grade level.

The consensus algorithm adapts to the number of available grade estimates:

| Estimates Available | Strategy | Rationale |
|---------------------|----------|-----------|
| 1 | Use directly | No alternatives to compare |
| 2 | Arithmetic mean | Split the difference |
| 3-4 | Median | Robust to a single outlier |
| 5+ | Trimmed mean (drop min and max) | Discards the most extreme estimates, averages the rest |

For a typical analysis with all formulas succeeding, 7 grade-level estimates are produced (Flesch-Kincaid, Gunning Fog, Coleman-Liau, SMOG, ARI, Dale-Chall grade midpoint, and Spache). The consensus drops the highest and lowest, then averages the remaining 5.

### Confidence Levels

Confidence reflects how many formulas produced valid results and how much they agree:

| Condition | Confidence | Meaning |
|-----------|------------|---------|
| 0 grade estimates | `low` | No formulas succeeded |
| 1-3 grade estimates | `medium` | Few formulas available to compare |
| 4+ estimates, spread ≤ 3 grades | `high` | Multiple formulas strongly agree |
| 4+ estimates, spread ≤ 6 grades | `medium` | Formulas moderately agree |
| 4+ estimates, spread > 6 grades | `low` | Formulas disagree significantly |

### Agreement and Stability

The response includes an `agreement` object showing how much the individual formulas agree:

| Spread (max - min grade) | Stability | Interpretation |
|--------------------------|-----------|----------------|
| 0-2 grades | `high` | Formulas strongly agree |
| 2-5 grades | `medium` | Typical for most texts |
| > 5 grades | `low` | Text has unusual characteristics (very short, mixed complexity, or non-prose) |

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

### Formula Definitions

Each formula is implemented from its published academic definition, not ported from another library. The exact equations:

**Flesch Reading Ease** (Flesch, 1948)
```
FRE = 206.835 − 1.015 × (words / sentences) − 84.6 × (syllables / words)
```
Unique among the formulas: higher scores mean easier text. Scores can exceed 100 (very simple text) or go negative (extremely dense academic prose). Most text falls between 30 and 70.

**Flesch-Kincaid Grade Level** (Kincaid et al., 1975)
```
FK = 0.39 × (words / sentences) + 11.8 × (syllables / words) − 15.59
```
Developed for the US Navy to assess training material readability. Maps directly to US school grade levels.

**Gunning Fog Index** (Gunning, 1952)
```
Fog = 0.4 × ((words / sentences) + 100 × (complex_words / words))
```
"Complex words" are words with 3 or more syllables. The canonical formula excludes proper nouns and words whose 3+ syllable count is due to common suffixes (-es, -ed, -ing); this implementation counts all 3+ syllable words, with the syllable counter itself handling silent -ed and sibilant -es adjustments. Designed to estimate the years of formal education needed.

**Coleman-Liau Index** (Coleman & Liau, 1975)
```
CLI = 0.0588 × L − 0.296 × S − 15.8
```
Where L = average letters per 100 words, S = average sentences per 100 words. Because it counts characters instead of syllables, it is faster to compute and independent of pronunciation rules.

**SMOG Index** (McLaughlin, 1969)
```
SMOG = 1.0430 × √(polysyllables × 30 / sentences) + 3.1291
```
Polysyllables are words with 3+ syllables. Designed for health literacy assessment. Most reliable with 30+ sentences; the API warns when fewer are available.

**Automated Readability Index** (Smith & Senter, 1967)
```
ARI = 4.71 × (characters / words) + 0.5 × (words / sentences) − 21.43
```
A character-based formula originally developed for real-time readability assessment on typewriters. Like Coleman-Liau, it avoids syllable counting entirely.

**Dale-Chall Readability Score** (Dale & Chall, 1948; revised 1995)
```
Raw = 0.1579 × (% difficult words) + 0.0496 × (words / sentences)
Adjusted = Raw + 3.6365  (if difficult word % > 5%)
```
"Difficult" means the word does not appear on the Dale-Chall list of ~3,000 words familiar to 4th-grade students. Returns both the raw unadjusted score and the adjusted score, plus a grade band mapping.

**Spache Readability Formula** (Spache, 1953; revised 1974)
```
Spache = 0.121 × (words / sentences) + 0.082 × (% unfamiliar words) + 0.659
```
Uses its own ~1,000-word list tuned for primary-grade (K-3) texts. Best used on material intended for young readers; scores on advanced text are informational.

## Warning Codes

| Code | Meaning |
|------|---------|
| `SHORT_TEXT` | Fewer than 100 words; scores are less stable |
| `FEW_SENTENCES` | Fewer than 3 sentences detected |
| `NO_SENTENCE_TERMINATOR` | No terminal punctuation found; fallback sentence construction used |
| `SMOG_SHORT_SAMPLE` | Fewer than 30 sentences; SMOG is less reliable |
| `PARTIAL_RESULT` | CPU budget exceeded; some metrics may be incomplete |

## How Text Processing Works

Every analysis request passes through a multi-stage pipeline before any formula runs. The pipeline explains why the same text can produce slightly different scores depending on `input_format` and why certain edge cases behave the way they do.

### 1. Input Format Detection

When `input_format` is `auto` (the default), the engine inspects the first 1,000 characters for structural clues:

- **HTML**: detected by `<!DOCTYPE`, `<html`, `<p>`, `<div>`, or `<h1>`-`<h6>` tags
- **Markdown**: detected by YAML frontmatter (`---`) or lines starting with `#`
- **Plain text**: the fallback when no markup is detected

You can override detection by setting `input_format` explicitly.

### 2. Preprocessing

Detected markup is stripped to produce clean prose:

- **HTML**: tags are removed, common entities are decoded (`&amp;`, `&lt;`, `&gt;`, `&quot;`, numeric entities like `&#8220;` and `&#x201C;`)
- **Markdown**: headings, links, emphasis, code fences, and other syntax are removed while preserving the readable text
- **Plain text**: whitespace is normalized and line endings are unified

### 3. Sentence Segmentation

Sentences are split on terminal punctuation (`.`, `?`, `!`) with awareness of common abbreviations (Dr., Mr., U.S., e.g., etc.) to avoid false splits. The segmenter handles:

- Repeated punctuation (`!!!`, `???`)
- Closing quotes and parentheses after terminal punctuation
- Text with no terminal punctuation (treated as a single sentence, with a warning)
- Non-empty text always produces at least one sentence

### 4. Word Tokenization

Words are split on whitespace and punctuation boundaries. Leading and trailing punctuation is stripped from each token. Empty tokens are filtered.

### 5. Syllable Counting

Syllables are counted using a multi-layered approach:

1. **LRU cache lookup** (2,048 entries): most English text reuses a small vocabulary (Zipf's law), so repeated words are served from cache
2. **Exception dictionary**: a hand-curated list of words whose syllable counts are frequently miscounted by rules (e.g., "business" = 2, not 3)
3. **Rule-based algorithm**: counts vowel groups, adjusts for silent-e, terminal -le, silent -ed, and sibilant -es endings. Treats 'y' as a vowel when between consonants.

Every word is guaranteed at least 1 syllable.

### 6. Word List Lookups

For Dale-Chall and Spache formulas, each word is checked against the respective "easy word" list using O(1) HashSet lookups. Words are matched with simple stemming: the engine strips common suffixes (-s, -es, -ed, -ing, -ly) and checks both the original and stemmed forms. A word is "difficult" only if neither form appears on the list.

### 7. Statistics Computation

Statistics are computed in tiers to minimize unnecessary work:

| Tier | Stats | Dependency |
|------|-------|------------|
| Tier 0 | Words, sentences, characters, paragraphs, unique words | Always computed |
| Tier 1 | Syllables, polysyllables, complex words | Requires syllable counting |
| Tier 2 | Dale-Chall difficult %, Spache unfamiliar % | Requires word list lookups |

All tiers are currently computed for every request, but the tiered architecture allows future optimization for selective formula execution.

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

### Crate Responsibilities

| Crate | Purpose | Platform |
|-------|---------|----------|
| `readability-types` | Shared data structures, error types, request/response schemas. Zero logic. | Any |
| `readability-core` | Analysis engine: formulas, tokenizer, syllable counter, word lists, preprocessor, consensus algorithm | Any (pure Rust, no I/O) |
| `readability-worker` | HTTP routing, authentication, CORS, ETag caching, request validation, response formatting | `wasm32-unknown-unknown` |
| `readability-cli` | File/stdin input, argument parsing, JSON output | Native |

### Data Flow

```
Input text
  │
  ├─ Format detection (auto/plain/html/markdown)
  │
  ├─ Preprocessing (strip markup → clean prose)
  │
  ├─ Tokenization (sentences → words)
  │
  ├─ Statistics (syllables, characters, word lists)
  │
  ├─ Formulas (8 formulas run over shared stats)
  │
  ├─ Consensus (trimmed mean of grade-level estimates)
  │
  ├─ Enrichment (paragraphs, hardest sentences, warnings)
  │
  └─ Response assembly (format selection, rounding)
```

## Design Principles

**Canonical formulas from academic sources.** Every formula is implemented from its published academic definition, not ported from textstat, readable.io, or any other library. The implementation acknowledges prior art (particularly the DaveChild/Text-Statistics metric family selection) but does not attempt compatibility with any specific library's quirks or bugs.

**Compute once, read many.** Text statistics (word counts, syllable totals, sentence boundaries) are computed once into a shared `TextStats` struct. All 8 formulas read from this shared struct rather than re-tokenizing or re-counting. This means formula implementations are pure functions that take stats and return a score.

**No NaN or Infinity by construction.** Rather than sanitizing outputs after the fact, the engine prevents NaN/Infinity at the source. The sentence segmenter guarantees at least 1 sentence for non-empty text. Every formula guards against division by zero (words == 0 or sentences == 0). The syllable counter guarantees at least 1 syllable per word. These invariants are enforced by property tests that throw arbitrary Unicode strings at the engine.

**Privacy by default.** Submitted text is never logged, stored, persisted, or included in error messages. This is enforced architecturally (the Worker has no KV, D1, R2, or Durable Object bindings) and verified by a dedicated privacy test that asserts the analysis output does not contain the input text.

**Deterministic.** Same text + same options = same response, byte for byte. This enables ETag-based HTTP caching and makes test assertions straightforward. There is no randomness, no timestamp-dependent logic, and no external state that could vary between requests.

**Edge-native.** The entire engine compiles to WebAssembly and runs on Cloudflare's edge network. Word lists and the syllable exception dictionary are embedded at compile time via `include_str!` in the build script, so there are no runtime file reads or network calls. The release profile uses `opt-level = "z"` (size optimization), LTO, and single codegen unit to minimize WASM binary size.

## Testing Strategy

The test suite is organized in layers, each targeting different classes of bugs:

### Golden Tests (19 tests)

Fixed-input, fixed-output tests where expected values are hand-calculated from the published formula definitions. Each formula has at least one golden test. These catch coefficient typos, sign errors, and rounding mistakes.

Five of these tests were specifically designed to kill mutants identified by `cargo-mutants`. These mutants survived the initial test suite because they produced numerically plausible but incorrect results (e.g., negating a formula, swapping coefficients).

### Snapshot Tests (10 snapshots)

Full JSON response snapshots via the `insta` crate, covering every fixture text with both default and full option sets. Any change to response structure, field names, numeric values, or serialization format breaks a snapshot. These catch unintentional regressions across the entire response surface.

### Property and Fuzz Tests (13 tests)

Proptest-based fuzz tests using 5 custom string generators (arbitrary UTF-8, ASCII, punctuation-only, whitespace-heavy, HTML-like) plus 8 deterministic edge cases (10,000-char single word, embedded null bytes, emoji text, mixed scripts, etc.). Every test asserts: no panics, no NaN, no Infinity, no negative grade levels (except for formulas that legitimately go negative).

### Cross-Library Validation

A Python comparison harness (`tests/cross-library/compare.py`) runs the CLI and Python's `textstat` library against the same 6 fixture texts and compares 7 metrics per text (42 comparisons). Per-metric thresholds account for known differences in syllable counting and word list implementations between the two libraries. All 42 comparisons pass within threshold.

### Mutation Testing

`cargo-mutants` systematically modifies formula code (changing operators, constants, return values) and verifies the test suite catches each mutation. 170 out of 171 mutants are caught; the single survivor is a mutation in unreachable code.

### Privacy Tests

A dedicated test verifies that the analysis output (serialized to JSON) does not contain the original input text, confirming the privacy guarantee.

## Project Structure

```
rusty-readability/
├── Cargo.toml                          # Workspace root
├── Cargo.lock                          # Pinned dependency versions
├── wrangler.toml                       # Cloudflare Worker configuration
├── openapi.json                        # OpenAPI 3.0.3 specification
│
├── crates/
│   ├── readability-types/              # Shared type definitions
│   │   └── src/
│   │       ├── lib.rs                  # Re-exports
│   │       ├── options.rs              # AnalyzeOptions, InputFormat, ResponseFormat
│   │       ├── request.rs              # AnalyzeRequest, BatchRequest
│   │       ├── response.rs             # AnalysisResult, Scores, PrimaryResult, etc.
│   │       └── errors.rs              # ReadabilityError, WarningCode, FormulaErrorCode
│   │
│   ├── readability-core/               # Analysis engine (pure Rust)
│   │   ├── build.rs                    # Embeds word lists at compile time
│   │   ├── src/
│   │   │   ├── analyze.rs              # Top-level analyze() and analyze_batch()
│   │   │   ├── consensus.rs            # Trimmed mean consensus algorithm
│   │   │   ├── stats.rs                # Tiered text statistics computation
│   │   │   ├── formulas/               # 8 readability formula implementations
│   │   │   ├── tokenize/               # Sentence segmenter + word tokenizer
│   │   │   ├── syllables/              # Rule-based counter + exceptions + LRU cache
│   │   │   ├── wordlists/              # Dale-Chall and Spache HashSet lookups
│   │   │   ├── preprocess/             # HTML, Markdown, plain text preprocessing
│   │   │   ├── paragraphs.rs           # Per-paragraph breakdown + hardest sentences
│   │   │   ├── warnings.rs             # Warning generation engine
│   │   │   ├── interpretation.rs       # Flesch ease labels, grade band labels
│   │   │   └── difficulty.rs           # Sentence difficulty classification
│   │   ├── tests/                      # Golden, property, snapshot, privacy, edge case tests
│   │   └── benches/                    # Criterion benchmarks
│   │
│   ├── readability-worker/             # Cloudflare Worker (WASM)
│   │   └── src/
│   │       ├── lib.rs                  # Request routing and Worker entry point
│   │       ├── routes.rs               # Endpoint handlers, response formatting, ETag
│   │       ├── auth.rs                 # csvkey validation with constant-time comparison
│   │       ├── cors.rs                 # CORS preflight and response headers
│   │       └── errors.rs              # Structured JSON error responses
│   │
│   └── readability-cli/                # Command-line interface
│       └── src/main.rs                 # File/stdin input, JSON output
│
├── data/                               # Source data (embedded at compile time)
│   ├── dale_chall_easy_words.txt       # ~3,700 word easy word list
│   ├── spache_easy_words.txt           # ~1,000 word easy word list
│   ├── syllable_exceptions.tsv         # Syllable count overrides
│   └── licenses/                       # Word list license files
│
├── fixtures/texts/                     # Test corpus (8 texts)
│
└── tests/
    ├── worker-integration/             # 72 curl-based integration tests
    └── cross-library/                  # Python comparison harness vs textstat
```

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
