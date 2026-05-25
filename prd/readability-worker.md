# PRD: Rust + Cloudflare Worker Readability Analysis API

**Status:** PRD v3.1  
**Last updated:** 2026-05-25  
**Primary objective:** Build a production-grade, edge-native readability analysis API written in Rust and deployed to Cloudflare Workers, implementing canonical readability formulas with corrected math, actionable feedback, and privacy-preserving design.

---

## 1. Executive Verdict

The final product should be a small, pure-Rust readability engine compiled to WebAssembly and exposed through a Cloudflare Worker. It should return one practical primary readability result, all individual formula scores, raw text statistics, confidence levels, warnings, formula metadata, tokenizer metadata, and word-list versions. The Worker should be stateless, deterministic, privacy-preserving, and safe to run at the edge.

The implementation uses canonical (academically correct) formulas throughout. DaveChild/Text-Statistics is acknowledged as prior art that informed the metric family selection, but no compatibility mode or legacy behavior is implemented. The formulas are implemented from their published academic definitions, not ported from any specific library.

```
A corrected, transparent, versioned readability-analysis API,
implemented as a reusable Rust engine and deployed as a Cloudflare Worker,
returning both individual readability formulas and a practical consensus score,
with explicit caveats, no raw-text logging, robust validation, and edge-friendly performance.
```

---

## 2. Goals

### 2.1 Product Goals

1. Accept a chunk of English text and return readability scores in a single API response.
2. Provide a simple primary result for product/UI use.
3. Preserve all individual formula-level details for transparency and compliance.
4. Implement canonical (academically correct) readability formulas.
5. Return raw counts so score differences are explainable.
6. Return warnings when formulas are being used outside ideal conditions.
7. Avoid storing or logging user text by default.
8. Stay small and fast enough for practical Cloudflare Worker use.
9. Keep the core readability engine independent from Cloudflare-specific code.
10. Provide actionable paragraph-level and sentence-level feedback, not just document-level scores.
11. Support batch analysis for SEO/content-quality pipelines.

### 2.2 Engineering Goals

1. Compute shared text statistics once, then run formulas over the shared stats.
2. Use pure functions for formula implementations.
3. Use deterministic English tokenization rather than heavyweight NLP.
4. Use compact static word-list lookups for Dale-Chall and Spache.
5. Use a carefully versioned syllable estimator.
6. Produce stable JSON responses with versioned schema and engine metadata.
7. Validate against fixed-count formula tests, fixture corpora, and cross-library comparisons.
8. Build and test both native Rust and `wasm32-unknown-unknown` targets.
9. Use demand-driven (lazy) stats computation — only compute what the requested metrics need.
10. Use arena allocation for per-request tokenizer work to minimize WASM allocation overhead.
11. Maintain a bounded cross-request syllable cache to amortize repeated vocabulary.

### 2.3 Non-Goals for v1

1. Multilingual readability scoring.
2. AI rewriting, summarization, or tone editing.
3. Full NLP parsing.
4. PDF/DOCX/EPUB upload parsing.
5. Persistent user accounts or billing.
6. Storing submitted text in KV, D1, R2, Durable Objects, logs, or analytics.
7. Perfect pronunciation-level syllable counting.
8. Certifying legal, medical, or educational compliance.

---

## 3. Success Criteria

| Area | Target |
|------|--------|
| p50 CPU time, 500-word plain text | ≤ 2 ms |
| p95 CPU time, 5,000-word plain text | ≤ 10 ms in optimized build |
| p95 CPU time, 25,000-word text on paid config | ≤ 50 ms |
| Free-plan default max body | 128 KB |
| Paid/default production max body | 256 KB, configurable |
| Compressed Worker size | ≤ 1 MB target, hard gate below platform plan limit |
| Startup-time validation | No expensive runtime dictionary parsing |
| Formula unit tests | 100% of supported formulas covered |
| Golden fixtures | ≥ 30 representative texts |
| Fuzz/property tests | No panics, no NaN/Inf in JSON |
| Privacy | No raw submitted text logged by default |
| API stability | Versioned endpoint and versioned engine metadata |
| Batch throughput | 10-text batch completes within 50ms CPU |
| Syllable cache hit rate | ≥ 70% on typical English prose (Zipf distribution) |

**Notes:**

- The 128 KB default is a product/runtime choice, not a Cloudflare request-body ceiling. It keeps CPU predictable on the free tier and avoids accidental expensive analysis.
- Larger limits can be enabled per deployment after benchmark results support them.

---

## 4. Product Positioning

This should be a readability analysis API, not merely a formula calculator.

A formula calculator returns a number. This product returns:

```
number + consensus + explanation + caveats + raw evidence + actionable breakdown + versioned implementation contract
```

The API should be usable by:

- content editors (paragraph-level feedback for targeted rewrites),
- SEO/content-quality tools (batch analysis across page inventories),
- documentation teams,
- education products,
- customer support QA tools,
- health communication tools,
- compliance reviewers,
- product teams scoring AI-generated copy,
- developers needing a fast readability scoring API.

A typical product-level answer should read like:

```
This text reads at approximately a 9th-grade level. Flesch Reading Ease is 58.3, 
which is fairly difficult. The biggest contributors are long sentences, high 
complex-word density, and Dale-Chall difficult vocabulary. Paragraph 3 is the 
hardest section (avg 29.7 words/sentence). Scores are medium confidence because 
the sample is under 30 sentences, so SMOG is reported with a short-sample warning.
```

---

## 5. Formula Specification

### 5.1 Supported Metrics

The API implements the following readability formulas from their canonical academic definitions:

- Flesch Reading Ease
- Flesch-Kincaid Grade Level
- Gunning Fog Index
- Coleman-Liau Index
- SMOG Index
- Automated Readability Index (ARI)
- Dale-Chall Readability Score
- Spache Readability Formula (revised)

### 5.2 Shared Variables

| Symbol | Meaning |
|--------|---------|
| `W` | total words |
| `S` | total sentences |
| `SYL` | total syllables |
| `LET` | letters, using the configured letter-count policy |
| `ARI_CHARS` | ARI character count, using the configured ARI policy |
| `ASL` | average sentence length = `W / S` |
| `ASW` | average syllables per word = `SYL / W` |
| `L100` | letters per 100 words = `100 * LET / W` |
| `S100` | sentences per 100 words = `100 * S / W` |
| `CW` | complex words for Gunning Fog |
| `PCW` | percentage complex words = `100 * CW / W` |
| `POLY` | polysyllables for SMOG |
| `DC_DIFF` | Dale-Chall difficult word occurrences |
| `DC_PDW` | percentage Dale-Chall difficult words = `100 * DC_DIFF / W` |
| `SP_UNFAM_UNIQUE` | unique Spache unfamiliar words |
| `SP_UNFAM_PCT` | percentage unique unfamiliar words for Spache |

### 5.3 Formula Definitions

| Metric | Formula | Notes |
|--------|---------|-------|
| Flesch Reading Ease | `206.835 - 1.015*ASL - 84.6*ASW` | Return raw score. Optional `normalized_0_100` only when requested. |
| Flesch-Kincaid Grade | `0.39*ASL + 11.8*ASW - 15.59` | Return raw grade level. No clamping by default. |
| Gunning Fog | `0.4 * (ASL + PCW)` | Complex-word policy is explicit and versioned. |
| Coleman-Liau | `0.0588*L100 - 0.296*S100 - 15.8` | Uses letters and sentences per 100 words. |
| SMOG | `1.043 * sqrt(POLY * 30 / S) + 3.1291` | The `3.1291` constant is added outside the multiplication. Warn below 30 sentences. |
| ARI | `4.71*(ARI_CHARS/W) + 0.5*(W/S) - 21.43` | ARI character count includes ASCII alphanumeric by default. Policy is explicit. |
| Dale-Chall | raw = `0.1579*DC_PDW + 0.0496*ASL`; add `3.6365` if `DC_PDW > 5` | Return both `raw_unadjusted` and adjusted score. |
| Spache | `0.121*ASL + 0.082*SP_UNFAM_PCT + 0.659` | Revised formula using percentage of unique unfamiliar words. Warn about early-grade scope. |

### 5.4 Additional Metrics (Cheap to Compute)

These metrics are derived from data already computed during tokenization and add significant product value at near-zero CPU cost:

| Metric | Formula | Value |
|--------|---------|-------|
| Type-Token Ratio (TTR) | `unique_words / total_words` | Vocabulary diversity indicator; low TTR suggests repetitive writing |
| Hapax Percentage | `words_appearing_once / total_words` | Lexical richness; useful for AI-content detection and style analysis |
| Reading Time | `words / 238` (Brysbaert 2019 meta-analysis) | Practical UX metric for content planning |

These are always computed (Tier 0 stats) and returned in `stats`. Reading time WPM is a named constant, not configurable in v1.

---

## 6. API Specification

### 6.1 Endpoint Summary

| Method | Path | Purpose |
|--------|------|---------|
| `POST` | `/v1/analyze` | Primary endpoint for scoring text. |
| `POST` | `/v1/batch` | Batch analysis (up to 10 texts per request). |
| `GET` | `/v1/metrics` | Supported metrics, formulas, versions, warnings. |
| `GET` | `/v1/version` | Build metadata, engine metadata, word-list versions. |
| `GET` | `/health` | Health check (no auth required). |
| `OPTIONS` | `/*` | CORS preflight (no auth required). |

### 6.2 Authentication

All endpoints except `/health` and `OPTIONS` require a `csvkey` query parameter. This follows the same pattern as `rusty-gateway`:

```
POST /v1/analyze?csvkey=<secret>
```

The key is stored as a Cloudflare Worker secret (`CSVKEY`) and validated using constant-time comparison. Invalid or missing keys return:

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Unauthorized"
  }
}
```

with HTTP 401.

Implementation:

```rust
pub fn validate_auth(provided: &str, expected: &str) -> Result<(), Response> {
    if !constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        return Err(json_error(401, "Unauthorized", "unauthorized"));
    }
    Ok(())
}
```

The `csvkey` secret is configured via `wrangler secret put CSVKEY`.

### 6.3 Request (Single Analysis)

```json
{
  "text": "This is the text to analyze.",
  "metrics": ["all"],
  "options": {
    "input_format": "auto",
    "language": "en-US",
    "primary_score": "consensus_grade",
    "response_format": "full",
    "round": 2,
    "normalize": false,
    "include_stats": true,
    "include_explanations": true,
    "include_paragraphs": false,
    "include_hardest_sentences": 0,
    "include_difficult_words": false,
    "include_debug": false,
    "hash_text": true
  }
}
```

### 6.4 Request (Batch Analysis)

```json
{
  "texts": [
    { "id": "page-1", "text": "First text to analyze." },
    { "id": "page-2", "text": "Second text to analyze." }
  ],
  "options": { ... }
}
```

Batch rules:
- Maximum 10 texts per request.
- Each text subject to the same size limit as single requests.
- Total request body limit: 1 MB.
- Shared `options` apply to all texts.
- Each result includes the client-provided `id` for correlation.
- If one text fails (e.g., empty), its result contains an error; other texts still succeed.

### 6.5 Request Fields

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `text` | string | required | UTF-8 text or HTML-like input. |
| `metrics` | array | `["all"]` | Can request specific metrics. Returning all is default. |
| `options.input_format` | enum | `auto` | `auto`, `plain`, `html`, `markdown_lite`. |
| `options.language` | enum | `en-US` | English only in v1. Non-English returns warning/error depending on strictness. |
| `options.primary_score` | enum | `consensus_grade` | Future-proofed. |
| `options.response_format` | enum | `full` | `full`, `compact`, or `scores_only`. Controls response verbosity. |
| `options.round` | integer | `2` | Display rounding only. Internal math uses full precision. |
| `options.normalize` | boolean | `false` | Adds clamped display values. Raw is always returned. |
| `options.include_stats` | boolean | `true` | Include raw counts. Ignored in `scores_only` mode. |
| `options.include_explanations` | boolean | `true` | Include formula descriptions and caveats. Ignored in `compact`/`scores_only`. |
| `options.include_paragraphs` | boolean | `false` | Include per-paragraph breakdown. |
| `options.include_hardest_sentences` | integer | `0` | Number of hardest sentences to return (0 = disabled, max 10). |
| `options.include_difficult_words` | boolean | `false` | Include capped difficult-word samples. |
| `options.include_debug` | boolean | `false` | Include tokenizer/syllable diagnostic summary. |
| `options.hash_text` | boolean | `true` | Returns SHA-256 hash for client correlation without logging content. |

### 6.6 Response Formats

**`full`** — Default. Complete response with all requested sections.

**`compact`** — Scores, primary result, stats, and warnings. No explanations, no engine metadata, no formula-level warnings. Approximately 40% smaller than full.

**`scores_only`** — Minimal payload for high-volume integrations:

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

### 6.7 Full Response

```json
{
  "schema_version": "1.0.0",
  "request_id": "req_01HX...",
  "engine": {
    "formula_version": "canonical_en_v1",
    "tokenizer_version": "english_tokenizer_v1",
    "syllable_version": "english_syllable_hybrid_v1",
    "character_count_policy": "ascii_alphanumeric_ari_v1",
    "word_lists": {
      "dale_chall": "dale_chall_3000_v1",
      "spache": "spache_revised_v1"
    }
  },
  "input": {
    "language": "en-US",
    "input_format_detected": "plain",
    "bytes": 1842,
    "text_hash": "sha256:...",
    "warnings": []
  },
  "stats": {
    "sentences": 14,
    "words": 286,
    "unique_words": 143,
    "letters": 1324,
    "ari_characters": 1398,
    "graphemes": 1511,
    "paragraphs": 3,
    "syllables": 421,
    "polysyllables": 37,
    "complex_words": 32,
    "dale_chall_difficult_words": 41,
    "dale_chall_difficult_percentage": 14.34,
    "spache_unique_unfamiliar_words": 29,
    "spache_unique_unfamiliar_percentage": 20.28,
    "average_words_per_sentence": 20.43,
    "average_syllables_per_word": 1.47,
    "average_characters_per_word": 4.89,
    "longest_sentence_words": 41,
    "type_token_ratio": 0.50,
    "hapax_percentage": 0.31,
    "reading_time_seconds": 72,
    "reading_time_wpm": 238
  },
  "scores": {
    "flesch_reading_ease": {
      "raw": 62.37,
      "score": 62.37,
      "normalized_0_100": null,
      "higher_is_easier": true,
      "interpretation": "standard / plain English",
      "valid": true,
      "confidence": "medium",
      "warnings": []
    },
    "flesch_kincaid_grade": {
      "raw": 8.21,
      "score": 8.21,
      "grade_band": "8th grade",
      "higher_is_easier": false,
      "valid": true,
      "confidence": "medium",
      "warnings": []
    },
    "gunning_fog": {
      "raw": 12.44,
      "score": 12.44,
      "grade_band": "12th grade / college entry",
      "valid": true,
      "confidence": "medium",
      "inputs": {
        "complex_word_policy": "complex_word_policy_v1"
      },
      "warnings": []
    },
    "coleman_liau": {
      "raw": 9.17,
      "score": 9.17,
      "grade_band": "9th grade",
      "valid": true,
      "confidence": "medium",
      "warnings": []
    },
    "smog": {
      "raw": 10.39,
      "score": 10.39,
      "grade_band": "10th grade",
      "valid": false,
      "confidence": "low",
      "warnings": [
        {
          "code": "SMOG_SHORT_SAMPLE",
          "message": "SMOG is most reliable on 30 or more sentences."
        }
      ]
    },
    "automated_readability_index": {
      "raw": 8.91,
      "score": 8.91,
      "ceiling_grade": 9,
      "grade_band": "9th grade",
      "valid": true,
      "confidence": "medium",
      "warnings": []
    },
    "dale_chall": {
      "raw_unadjusted": 4.18,
      "raw": 7.82,
      "score": 7.82,
      "grade_band": "9th to 10th grade",
      "grade_midpoint": 9.5,
      "valid": true,
      "confidence": "medium",
      "warnings": []
    },
    "spache": {
      "raw": 4.98,
      "score": 4.98,
      "grade_band": "approximately 5th grade",
      "valid": false,
      "confidence": "low",
      "warnings": [
        {
          "code": "SPACHE_SCOPE",
          "message": "Spache is intended primarily for early-grade texts."
        }
      ]
    }
  },
  "primary": {
    "type": "consensus_grade",
    "score": 8.9,
    "label": "approximately 9th grade",
    "ease_score": 62.37,
    "ease_label": "standard / plain English",
    "confidence": "medium",
    "agreement": {
      "min_grade": 8.21,
      "max_grade": 12.44,
      "spread": 4.23,
      "stability": "medium"
    },
    "included_metrics": [
      "flesch_kincaid_grade",
      "gunning_fog",
      "coleman_liau",
      "automated_readability_index",
      "dale_chall"
    ],
    "excluded_metrics": [
      {
        "metric": "smog",
        "reason": "short_sample"
      },
      {
        "metric": "spache",
        "reason": "outside_recommended_scope"
      }
    ],
    "drivers": [
      {
        "factor": "sentence_length",
        "direction": "harder",
        "evidence": "20.43 average words per sentence"
      },
      {
        "factor": "complex_words",
        "direction": "harder",
        "evidence": "32 complex words (11.2%)"
      }
    ],
    "summary": "The text is moderately difficult. Sentence length and complex-word density are the main contributors."
  },
  "paragraphs": [
    {
      "index": 0,
      "words": 92,
      "sentences": 4,
      "avg_sentence_length": 23.0,
      "avg_syllables_per_word": 1.52,
      "difficulty": "moderate"
    },
    {
      "index": 1,
      "words": 105,
      "sentences": 5,
      "avg_sentence_length": 21.0,
      "avg_syllables_per_word": 1.41,
      "difficulty": "moderate"
    },
    {
      "index": 2,
      "words": 89,
      "sentences": 5,
      "avg_sentence_length": 17.8,
      "avg_syllables_per_word": 1.49,
      "difficulty": "easy"
    }
  ],
  "hardest_sentences": [
    {
      "index": 7,
      "paragraph_index": 1,
      "words": 41,
      "syllables_per_word": 1.9,
      "complex_word_count": 6,
      "preview": "The implementation of distributed consensus algorithms requires..."
    }
  ],
  "warnings": [
    {
      "code": "SHORT_TEXT",
      "message": "Scores are less stable below 100 words.",
      "severity": "info"
    }
  ]
}
```

### 6.8 Batch Response

```json
{
  "schema_version": "1.0.0",
  "request_id": "req_01HX...",
  "results": [
    {
      "id": "page-1",
      "status": "ok",
      "data": { ... }
    },
    {
      "id": "page-2",
      "status": "error",
      "error": {
        "code": "EMPTY_TEXT",
        "message": "No analyzable words in the provided text."
      }
    }
  ]
}
```

### 6.9 ETag Support

The API is deterministic: same text + same options = same response. The Worker returns an `ETag` header on every successful response:

```
ETag: "v1-<sha256_of_text>-<sha256_of_options>"
```

Clients may send `If-None-Match` on subsequent requests. If the ETag matches, the Worker returns `304 Not Modified` without re-running analysis. This is a major performance win for the "re-check after editing" workflow — clients repeatedly analyzing the same text (e.g., draft auto-save) get instant responses.

Implementation: compute the ETag from the text hash + serialized options hash early in the request pipeline, before full analysis. Compare against `If-None-Match` header. Short-circuit on match.

---

## 7. Primary Score Design

### 7.1 Why Consensus is Needed

No single readability formula is reliable enough to be the only product answer. Flesch Reading Ease is intuitive but not a grade level. SMOG is useful but short-sample sensitive. Spache is scoped to early-grade readers. Dale-Chall depends on a word list. Coleman-Liau does not use syllables and can be useful when syllable confidence is lower.

Therefore the API should expose all formulas but present one primary result:

```
primary.consensus_grade
```

### 7.2 Consensus Algorithm

1. Compute all grade-like metrics:
   - Flesch-Kincaid Grade,
   - Gunning Fog,
   - Coleman-Liau,
   - SMOG,
   - ARI,
   - Dale-Chall mapped to grade-band midpoint,
   - Spache only when early-grade applicability is true.

2. Exclude invalid metrics (those that returned errors or have `valid: false`).
3. Exclude SMOG from the default consensus when fewer than 30 sentences.
4. Exclude Spache when the text is outside early-grade scope.
5. Convert Dale-Chall score to grade-band midpoint before consensus because Dale-Chall raw scores are not direct grade levels.
6. Sort included grade estimates ascending.
7. Apply aggregation based on count of remaining metrics:
   - **5+ metrics:** drop min and max, compute arithmetic mean of the remaining values.
   - **3–4 metrics:** compute median (middle value; if even count, average of middle two).
   - **2 metrics:** compute arithmetic mean.
   - **1 metric:** use it directly with `"confidence": "low"`.
   - **0 metrics:** return `null` primary score with `NO_VALID_METRICS` warning.
8. Return metric disagreement in the `agreement` object:

```json
"agreement": {
  "min_grade": 7.8,
  "max_grade": 12.4,
  "spread": 4.6,
  "stability": "medium"
}
```

Stability thresholds:
- `"high"`: spread ≤ 2.0
- `"medium"`: spread ≤ 5.0
- `"low"`: spread > 5.0

9. Confidence rules:

| Confidence | Conditions |
|------------|-----------|
| `high` | ≥ 300 words, ≥ 10 sentences, ≥ 5 valid metrics, agreement stability = high. |
| `medium` | ≥ 100 words, ≥ 3 sentences, ≥ 3 valid metrics, agreement stability ≤ medium. |
| `low` | Short text, few sentences, high disagreement, non-English likely, code-heavy input, or mostly rule-based syllable estimates. |

---

## 8. Text Processing Requirements

### 8.1 Preprocessing

The analyzer must support four input modes:

```
auto | plain | html | markdown_lite
```

`auto` is the default. Detection heuristics (applied in order):
1. Starts with `<!DOCTYPE` or `<html` (case-insensitive) → `html`.
2. Contains `<p>`, `<div>`, or `<h1`–`<h6` tags → `html`.
3. Starts with `---\n` (YAML frontmatter) or first non-blank line starts with `#` → `markdown_lite`.
4. Otherwise → `plain`.

**Plain-text preprocessing:**

1. Normalize Unicode quotes and dashes.
2. Preserve paragraph boundaries (double newlines).
3. Collapse excessive whitespace.
4. Preserve sentence-ending punctuation before token cleanup.
5. Strip control characters except useful whitespace.

**HTML preprocessing:**

1. Remove `<script>`, `<style>`, `<noscript>`, and comments.
2. Convert block elements to sentence/paragraph boundaries:
   - `p`, `div`, `li`, `h1`–`h6`, `br`, `section`, `article`, `tr`, `td`.
3. Decode common HTML entities.
4. Strip remaining tags.
5. Preserve visible text order as best as possible without using a heavy DOM dependency.
6. Return `HTML_MALFORMED` warning if cleanup is uncertain.

**Markdown-lite preprocessing:**

1. Strip frontmatter boundaries (`---`).
2. Treat headings as sentence-like units.
3. Treat list items as sentence-like units.
4. Ignore fenced code blocks by default, with option `include_code_blocks: true`.
5. Strip link URLs but preserve link text.
6. Strip image references.
7. Preserve table cell content as sentence-like units.

### 8.2 Sentence Segmentation

Rules:

1. Split on `.`, `?`, `!`, and repeated terminal punctuation.
2. Do not split common abbreviations:
   - `Mr.`, `Mrs.`, `Ms.`, `Dr.`, `Prof.`, `Sr.`, `Jr.`, `St.`, `vs.`, `etc.`, `e.g.`, `i.e.`, `Inc.`, `Ltd.`, `a.m.`, `p.m.`
3. Do not split initials or acronyms:
   - `U.S.`, `U.K.`, `A.I.`, `F. Scott Fitzgerald`.
4. Do not split decimals:
   - `3.14`, `1,000.50`.
5. Treat headings and list items as sentence-like units when no terminal punctuation exists.
6. Ensure a non-empty analyzable text has at least one sentence.
7. Return `NO_SENTENCE_TERMINATOR` warning when fallback sentence construction was used.

### 8.3 Word Tokenization

Rules:

1. A word is a lexical token containing at least one letter.
2. Contractions count as one word:
   - `don't`, `can't`, `we're`.
3. Hyphenated compounds default to one user-facing word but can be inspected internally for Gunning Fog exclusions.
4. Pure numbers do not contribute syllables.
5. Numbers can contribute to ARI character counts depending on `character_count_policy`.
6. Original casing is preserved for proper-noun heuristics.
7. Lowercase normalized form is used for dictionary/word-list lookup.
8. Curly apostrophes are normalized.

---

## 9. Shared Stats Model — Tiered Computation

### 9.1 Computation Tiers

Stats are computed on-demand based on which metrics the client requests. This avoids expensive word-list lookups when only character-based metrics are needed.

| Tier | Stats | Triggers |
|------|-------|----------|
| **Tier 0** (always) | bytes, words, sentences, paragraphs, letters, ari_characters, graphemes, unique_words, longest_sentence_words, type_token_ratio, hapax_percentage, reading_time | Any request |
| **Tier 1** (syllable-dependent) | syllables, polysyllables, complex_words, average_syllables_per_word | Flesch, Flesch-Kincaid, Gunning Fog, SMOG, or `metrics: ["all"]` |
| **Tier 2** (word-list-dependent) | dale_chall_difficult_words, dale_chall_difficult_percentage, spache_unique_unfamiliar_words, spache_unique_unfamiliar_percentage | Dale-Chall, Spache, or `metrics: ["all"]` |

Example: A request for `metrics: ["coleman_liau", "ari"]` computes only Tier 0 stats — no syllable counting, no word-list lookups.

### 9.2 Struct Definition

```rust
pub struct TextStats {
    // Tier 0: always computed
    pub bytes: usize,
    pub graphemes: usize,
    pub letters_ascii: usize,
    pub letters_unicode: usize,
    pub ari_characters: usize,
    pub words: usize,
    pub unique_words: usize,
    pub hapax_count: usize,
    pub sentences: usize,
    pub paragraphs: usize,
    pub average_words_per_sentence: f64,
    pub average_characters_per_word: f64,
    pub longest_word_len: usize,
    pub longest_sentence_words: usize,

    // Tier 1: syllable-dependent (Option = not yet computed)
    pub syllables: Option<usize>,
    pub polysyllables: Option<usize>,
    pub complex_words: Option<usize>,
    pub complex_words_excluding_proper: Option<usize>,
    pub average_syllables_per_word: Option<f64>,
    pub syllable_estimates_from_dictionary: Option<usize>,
    pub syllable_estimates_from_rules: Option<usize>,

    // Tier 2: word-list-dependent
    pub dale_chall_difficult_words: Option<usize>,
    pub dale_chall_difficult_percentage: Option<f64>,
    pub spache_unique_unfamiliar_words: Option<usize>,
    pub spache_unique_unfamiliar_percentage: Option<f64>,
}
```

All formulas consume `TextStats`. No formula should re-tokenize the text.

---

## 10. Syllable Counting Strategy

### 10.1 Recommended Implementation

Use a layered syllable estimator with a cross-request cache:

```
syllables(word):
  1. normalize word
  2. if in cross-request LRU cache: return cached value
  3. if in exception map: cache and return exception value
  4. if optional pronunciation dictionary feature is enabled and word exists: cache and return
  5. compute rule-based fallback
  6. cache result
  7. return max(1, count)
```

### 10.2 Cross-Request LRU Cache

Workers reuse isolates across requests. A bounded LRU cache (2048 entries) persists syllable counts across requests within the same isolate, amortizing computation for repeated vocabulary. English has a Zipfian word distribution — the top 2000 words cover approximately 80% of running text.

Implementation: `static RefCell<LruCache<String, u8>>` (single-threaded WASM). Benchmark to ensure the cache lookup cost does not exceed rule-based recomputation for short words.

### 10.3 v1 Default

For v1, ship:

1. Exception dictionary (hardcoded irregular words).
2. Rule-based fallback.
3. Cross-request bounded LRU cache (2048 entries).
4. Versioned syllable algorithm.
5. Warning when rule-based estimates dominate (>80% of words).

### 10.4 Optional Pronunciation Dictionary Feature

A high-frequency CMU Pronouncing Dictionary subset is valuable, but it should be behind a build-time feature flag until licensing, bundle size, and accuracy benchmarks are complete.

Feature:

```toml
[features]
default = ["small-syllable"]
cmu-syllable = []
```

Rules:

1. CMU subset must pass license review.
2. Bundle-size impact must be measured in CI.
3. Syllable accuracy must improve materially on benchmark corpora.
4. If enabled, response must say:

```
"syllable_version": "english_syllable_cmu20k_plus_rules_v1"
```

### 10.5 Fallback Rules

The rule-based estimator should include:

- vowel-group counting,
- silent `e` handling,
- terminal `le` handling,
- `-ed`, `-es`, `-ing` adjustments,
- diphthong handling,
- prefix/suffix adjustment where supported by tests,
- minimum one syllable per alphabetic word.

### 10.6 Syllable Tests

Fixture categories:

- simple words: `cat`, `dog`, `reading`, `computer`,
- silent-e words: `make`, `time`, `cake`,
- terminal-le words: `table`, `little`,
- irregulars: `business`, `queue`, `people`, `beautiful`, `colonel`, `Wednesday`, `coyote`, `apostrophe`, `recipe`,
- polysyllabic words: `communication`, `international`, `readability`,
- proper nouns and technical terms.

---

## 11. Dale-Chall and Spache Word-List Requirements

### 11.1 Source and Licensing

Before implementation begins, lock the word-list source and license strategy.

Rules:

1. Store canonical source files under `data/`.
2. Include license and provenance metadata for each list.
3. Generate static lookup structures at build time via `build.rs`.
4. Return word-list versions in every response.
5. Do not copy GPL code or data into a permissively licensed/commercial codebase unless legal review permits it.

### 11.2 Lookup Strategy

Preferred implementation:

1. Store lowercase canonical words.
2. Generate common inflections at build time:
   - `s`, `es`, `ed`, `ing`, `ly`, plus safe irregular cases.
3. Use `phf::Set` or sorted static arrays with binary search.
4. Do not run a pluralizer per token at runtime unless benchmarks prove it acceptable.

### 11.3 Dale-Chall Rules

A word counts as Dale-Chall difficult when:

1. It has at least two alphabetic characters.
2. Its normalized form is not in the Dale-Chall familiar-word set.
3. Repetitions count.

Return:

```json
"dale_chall": {
  "raw_unadjusted": 4.18,
  "score": 7.82,
  "difficult_words": 41,
  "difficult_word_percentage": 14.34,
  "grade_band": "9th to 10th grade",
  "grade_midpoint": 9.5
}
```

### 11.4 Spache Rules

A word counts as Spache unfamiliar when:

1. It has at least two alphabetic characters.
2. Its normalized form is not in the Spache familiar-word set.
3. It has not already been counted in the same text.

The formula uses percentage of unique unfamiliar words, not raw count.

---

## 12. Architecture

### 12.1 Workspace Layout

```
rusty-readability/
  Cargo.toml              # virtual workspace
  wrangler.toml
  README.md
  LICENSE
  prd/
    readability-worker.md

  crates/
    readability-core/
      Cargo.toml
      build.rs            # word-list codegen (phf generation from data/)
      src/
        lib.rs
        analyze.rs
        options.rs
        stats.rs
        preprocess/
          mod.rs
          detect.rs       # auto-detection heuristics
          plain.rs
          html.rs
          markdown_lite.rs
        tokenize/
          mod.rs
          sentence.rs
          word.rs
          abbreviations.rs
        syllables/
          mod.rs
          cache.rs        # cross-request LRU
          rules.rs
          exceptions.rs
        difficulty.rs
        formulas/
          mod.rs
          flesch.rs
          flesch_kincaid.rs
          gunning_fog.rs
          coleman_liau.rs
          smog.rs
          ari.rs
          dale_chall.rs
          spache.rs
        consensus.rs
        interpretation.rs
        warnings.rs
        paragraphs.rs     # paragraph-level breakdown
        wordlists/
          mod.rs
          dale_chall.rs
          spache.rs
      tests/
        formula_golden.rs
        tokenizer.rs
        syllables.rs
        wordlists.rs
        consensus.rs
        paragraphs.rs

    readability-types/
      Cargo.toml
      src/
        lib.rs            # shared request/response types
        request.rs
        response.rs
        options.rs
        errors.rs

    readability-worker/
      Cargo.toml
      src/
        lib.rs
        validation.rs     # csvkey auth (mirrors rusty-gateway pattern)
        routes.rs
        cors.rs
        errors.rs
        logging.rs

    readability-cli/
      Cargo.toml
      src/main.rs         # first-class dev tool for testing, corpus generation

  data/
    dale_chall_easy_words.txt
    spache_easy_words.txt
    syllable_exceptions.tsv
    cmu_subset.tsv               # optional feature only
    licenses/
      dale_chall.LICENSE
      spache.LICENSE
      cmu.LICENSE

  scripts/
    compare_against_textstat.py
    corpus_report.py

  fixtures/
    texts/
      simple.txt
      complex.txt
      html.html
      markdown.md
      abbreviations.txt
      short.txt
      legal.txt
      academic.txt
      health.txt
      children.txt
    expected/
      canonical/*.json
```

### 12.2 Crate Dependency Graph

```
readability-types   (no deps on other workspace crates)
       ↑
readability-core    (depends on: readability-types)
       ↑
readability-worker  (depends on: readability-core, readability-types)
readability-cli     (depends on: readability-core, readability-types)
```

The `readability-types` crate contains all request/response/option structs with serde derives. This enables:
- Future TypeScript SDK generation via `ts-rs` or `specta`
- OpenAPI spec generation from the type definitions
- Clean import boundary between worker and core

### 12.3 Core Crate Contract

```rust
pub fn analyze(input: &str, options: &AnalyzeOptions) -> Result<AnalysisResult, AnalyzeError>;
pub fn analyze_batch(inputs: &[(&str, &str)], options: &AnalyzeOptions) -> Vec<Result<AnalysisResult, AnalyzeError>>;
```

The core crate must have no Cloudflare-specific dependency. This enables:

1. Fast native unit tests.
2. CLI use.
3. Golden corpus scripts.
4. Worker deployment.
5. Future library reuse.

Each formula returns `Result<Score, FormulaError>`. A formula that errors (e.g., division by zero on degenerate input) does not cause the entire analysis to fail — it reports `"valid": false` with an error code, and remaining formulas proceed normally.

### 12.4 Worker Crate Responsibilities

The Worker crate should only:

1. Extract `csvkey` from query params and validate against `CSVKEY` secret.
2. Parse JSON request body.
3. Validate method/path/content type.
4. Enforce size limits (before full body parsing).
5. Normalize options.
6. Call `readability_core::analyze` or `analyze_batch`.
7. Serialize JSON response in the requested format.
8. Compute and check ETag for 304 responses.
9. Add CORS headers.
10. Emit safe logs.
11. Return structured errors with `request_id`.

No formula logic belongs in the Worker crate.

### 12.5 CLI Crate Responsibilities

The CLI is a first-class developer tool. It provides:

1. Single-file analysis with all output formats.
2. Corpus-wide analysis with CSV/JSON output.
3. Golden-test generation and comparison.
4. Syllable accuracy benchmarking against known-correct counts.
5. Cross-library comparison report generation.
6. stdin piping for integration with shell workflows.

---

## 13. Rust and Cloudflare Implementation

### 13.1 Recommended Dependencies

**Types crate:**

```
serde
serde_json
```

**Core:**

```
serde
serde_json
thiserror
phf (build-time generation)
bumpalo (arena allocator for per-request tokenization)
lru (bounded cross-request syllable cache)
sha2 (text hashing)
regex-lite (bounded preprocessing only)
```

**Worker:**

```
worker
wasm-bindgen
console_error_panic_hook
serde
serde_json
```

**CLI:**

```
clap
serde_json
csv (for corpus reports)
```

**Avoid:**

```
large NLP crates
native filesystem dependencies in core
native TLS stacks
runtime dictionary downloads
unbounded regex behavior
heavy HTML parsers unless bundle tests pass
unicode-segmentation (unless benchmarked; ASCII-only fast path preferred)
```

### 13.2 Build Profile

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = "debuginfo"
```

### 13.3 Wrangler Configuration

```toml
name = "rusty-readability"
main = "build/worker/shim.mjs"
compatibility_date = "2026-05-25"

[build]
command = "curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable && . \"$HOME/.cargo/env\" && rustup target add wasm32-unknown-unknown && cargo install -q worker-build && worker-build --release"

[vars]
READABILITY_VERSION = "1.0.0"
READABILITY_MAX_BYTES = "131072"
READABILITY_BATCH_MAX_BYTES = "1048576"
READABILITY_BATCH_MAX_TEXTS = "10"
READABILITY_LOG_LEVEL = "info"
```

Secrets (configured via `wrangler secret put`):

```
CSVKEY          # authentication token for all non-health endpoints
```

---

## 14. Input Limits and Runtime Policy

### 14.1 Defaults

| Limit | Default |
|-------|---------|
| Max single request body | 128 KB |
| Max batch request body | 1 MB |
| Max texts per batch | 10 |
| Min words for full confidence | 100 |
| Preferred minimum for SMOG | 30 sentences |
| Max difficult words returned | 100 |
| Max hardest sentences returned | 10 |
| Hardest sentence preview length | 80 characters |
| CPU budget per request | 30 ms (return partial on exceed) |

### 14.2 Size Limit Enforcement

Size limits are enforced **before** full body parsing:

1. Read `Content-Length` header.
2. If present and over limit: reject with 413 immediately (zero body read).
3. If absent (chunked): read stream up to `limit + 1` bytes. If exceeded, reject with 413.
4. Only after size validation: parse JSON body.

This prevents allocation of oversized strings and ensures CPU budget is not consumed by parsing alone.

### 14.3 CPU Budget and Partial Results

If analysis exceeds the configured CPU budget mid-computation:

1. Return whichever metric scores have completed.
2. Mark incomplete metrics as `"valid": false, "error": "TIMEOUT"`.
3. Add top-level warning: `PARTIAL_RESULT`.
4. Set HTTP status to 200 (not 500 — partial success is still useful).
5. Log the timeout event with input byte length and word count.

### 14.4 Why Not Accept Platform Maximums by Default?

Cloudflare request-body limits are much larger than this product needs, but readability analysis is CPU-bound and tokenization-heavy. The API should enforce a product-specific limit lower than the platform maximum to avoid CPU spikes, oversized JSON responses, and accidental misuse.

---

## 15. Warning Codes

| Code | Trigger | Severity |
|------|---------|----------|
| `EMPTY_TEXT` | No analyzable words. | error |
| `SHORT_TEXT` | Fewer than 100 words. | info |
| `FEW_SENTENCES` | Fewer than 3 sentences. | warning |
| `NO_SENTENCE_TERMINATOR` | Text has words but no terminal punctuation. | info |
| `SMOG_SHORT_SAMPLE` | Fewer than 30 sentences. | warning |
| `SPACHE_SCOPE` | Spache likely outside intended early-grade use. | warning |
| `NON_ENGLISH_UNSUPPORTED` | Non-English requested or likely. | warning/error |
| `HTML_MALFORMED` | HTML cleanup uncertain. | info |
| `MARKDOWN_CODE_IGNORED` | Code block skipped. | info |
| `TOKENIZER_AMBIGUITY` | Abbreviation/acronym segmentation uncertain. | info |
| `SYLLABLE_ESTIMATE` | Syllable count relies heavily on rules (>80%). | info |
| `NORMALIZED_SCORE` | Client requested clamped normalized values. | info |
| `DEBUG_OUTPUT_CAPPED` | Debug word/token output capped. | info |
| `TEXT_HASH_DISABLED` | Client or config disabled hash output. | info |
| `PARTIAL_RESULT` | CPU budget exceeded; some metrics incomplete. | warning |
| `NO_VALID_METRICS` | All metrics failed; no consensus possible. | error |
| `FORMULA_ERROR` | A specific formula produced an error (see per-metric). | warning |

---

## 16. Error Handling

### 16.1 Status Codes

| Status | Use |
|--------|-----|
| `200` | Successful analysis, including partial results and short text with warnings. |
| `304` | ETag match — content unchanged. |
| `400` | Invalid JSON, missing `text`, invalid options. |
| `401` | Missing or invalid `csvkey`. |
| `413` | Payload too large. |
| `415` | Unsupported content type. |
| `422` | Valid request but no analyzable words. |
| `500` | Internal error (including panic recovery). |

### 16.2 Error Shape

```json
{
  "error": {
    "code": "PAYLOAD_TOO_LARGE",
    "message": "Text exceeds the configured maximum size of 131072 bytes.",
    "request_id": "req_01HX..."
  }
}
```

Rules:

1. Never include raw submitted text in error messages.
2. Always include `request_id` (in both success and error responses).
3. Do not leak stack traces.
4. Do not serialize `NaN` or `Infinity`.

### 16.3 Per-Formula Error Isolation

Each formula returns `Result<Score, FormulaError>`. Possible formula errors:

- `DIVISION_BY_ZERO`: Input has zero sentences or zero words (degenerate after preprocessing).
- `TIMEOUT`: CPU budget exhausted before this formula completed.
- `INTERNAL`: Unexpected error in formula computation.

A failed formula appears in the response as:

```json
"smog": {
  "valid": false,
  "error": "DIVISION_BY_ZERO",
  "warnings": [...]
}
```

The consensus algorithm excludes errored metrics. Other formulas still return normally.

### 16.4 Panic Recovery

Wrap the `analyze()` call at the Worker boundary in `std::panic::catch_unwind`. If a panic escapes:

1. Return HTTP 500 with `INTERNAL_PANIC` error code.
2. Log: input byte length, input text hash, panic message (sanitized — strip any user text substrings).
3. Do not include user text in the error response or logs.

---

## 17. Privacy and Logging

### 17.1 Privacy Defaults

The Worker must not log or persist submitted text.

**Allowed logs:**

```
request_id
timestamp
path
status
duration_ms
byte_length
word_count
sentence_count
metrics_requested
warning_codes
error_code
etag_hit (boolean)
batch_size
```

**Disallowed logs:**

```
raw text
HTML content
snippets
full tokens
difficult-word list
user-provided excerpts
csvkey value
```

### 17.2 Text Hash

Return a SHA-256 hash by default unless disabled:

```
"text_hash": "sha256:..."
```

The hash helps clients correlate results without the service storing text. The Worker does not store the hash.

### 17.3 Request ID

Every response (success or error) includes:
- `request_id` field in the JSON body.
- `X-Request-Id` response header.

Generated via a compact random ID at the start of request handling.

---

## 18. Observability

### 18.1 Metrics

Track via `console_log!` structured logs (parseable by Cloudflare Logpush):

```
requests_total
errors_total (by error_code)
latency_ms (histogram)
payload_size_bytes
words_analyzed
batch_size
etag_hits
warning_code_counts
partial_result_count
auth_failures
worker_bundle_size_gzip
```

### 18.2 Safe Log Example

```json
{
  "request_id": "req_abc",
  "path": "/v1/analyze",
  "status": 200,
  "duration_ms": 4,
  "bytes": 1842,
  "words": 286,
  "sentences": 14,
  "metrics": ["all"],
  "response_format": "full",
  "etag_hit": false,
  "warnings": ["SMOG_SHORT_SAMPLE"]
}
```

### 18.3 No-Text-Logging Test

Add a test that submits a sentinel phrase and fails if the phrase appears in captured logs, errors, or analytics payloads in test mode.

---

## 19. Testing and Validation Plan

### 19.1 Formula Golden Tests

Each formula gets fixed-count tests independent of tokenization.

Example:

```rust
#[test]
fn smog_uses_correct_parentheses() {
    let stats = Stats {
        sentences: 30,
        polysyllables: 100,
        ..fixture()
    };

    let score = smog(&stats).unwrap();

    assert_approx_eq!(score, 1.043 * 100_f64.sqrt() + 3.1291, 1e-9);
}
```

Required cases:

1. Correct formula result.
2. Rounding behavior.
3. Normalized optional behavior.
4. Division-by-zero returns `Err(FormulaError)`, not panic.
5. Warning behavior.

### 19.2 Cross-Library Comparison

Compare against `textstat`, `py-readability-metrics`, and at least one formula-only implementation where appropriate.

Do not blindly require exact text-level equality because tokenizers and syllable estimators differ. Use two categories:

```
same formula + same fixed counts = exact match
same text + different tokenizer = explainable difference
```

### 19.3 Golden Corpus

Use at least 30 fixture texts:

- simple paragraph,
- children's text,
- academic abstract,
- legal-style text,
- health communication text,
- documentation page,
- marketing copy,
- support article,
- press release,
- recipe,
- poem,
- HTML page fragment,
- Markdown doc,
- code-heavy doc,
- abbreviation-heavy text,
- short UI copy,
- long article.

For each fixture, store:

```json
{
  "counts": {},
  "scores": {},
  "warnings": []
}
```

### 19.4 Snapshot Tests

Use `insta` for full API response snapshot testing. Every golden fixture gets a snapshot file checked into git. This catches:
- Unintentional schema drift (new fields, removed fields).
- Changed field ordering.
- Type changes (integer → float).
- Regression in score values.

Snapshot updates require explicit `cargo insta review` approval.

### 19.5 Tokenizer Tests

Fixtures:

1. `Mr. Smith went to Washington.`
2. `The U.S. economy grew.`
3. `Dr. Jones wrote e.g. examples.`
4. `Version 3.14 is stable.`
5. `This is a list:\n- First item\n- Second item`
6. `Hello!!! Are you there???`
7. `A.I. tools are common.`
8. HTML with `<p>`, `<li>`, `<script>`, and entities.
9. Markdown links, headings, and code fences.
10. Text with emojis and smart quotes.

### 19.6 Syllable Tests

Include:

- rule fixtures,
- exception fixtures,
- cross-request cache hit/miss verification,
- optional CMU subset accuracy test,
- hard-word regression list.

### 19.7 Property and Fuzz Tests

Use property/fuzz testing for:

1. No panics on arbitrary UTF-8.
2. No infinite loops.
3. No `NaN` or `Infinity` in response.
4. Non-negative counts.
5. Empty/punctuation-only text yields error or low-confidence warning.
6. Very long single-token input stays O(n).
7. Regexes do not catastrophically backtrack.
8. Batch with mixed valid/invalid texts does not cross-contaminate.

### 19.8 Mutation Tests

Use `cargo-mutants` in CI for formula code. Formula functions are pure and small — mutations (sign flips, constant changes, boundary shifts) must be caught by existing tests. Any surviving mutant indicates insufficient test coverage for that formula.

### 19.9 Worker Integration Tests

Use Wrangler/Miniflare-compatible testing to verify:

1. Valid plain text request with valid csvkey.
2. Missing csvkey → 401.
3. Invalid csvkey → 401.
4. Valid HTML request.
5. Missing text → 400.
6. Empty text → 422.
7. Payload too large → 413.
8. Unsupported content type → 415.
9. Specific metric subset (verify lazy computation skips Tier 1/2).
10. CORS preflight (no auth required).
11. Health endpoint (no auth required).
12. ETag hit → 304.
13. Batch with mixed success/error.
14. `scores_only` response format.
15. No raw text in logs.
16. `X-Request-Id` header present on all responses.

### 19.10 Performance Tests

Use `criterion` for native benchmarks and Worker-level smoke benchmarks.

**Scenarios:**

| Fixture | Target |
|---------|--------|
| 100-word short copy | p95 ≤ 2 ms CPU |
| 500-word article section | p95 ≤ 5 ms CPU |
| 5,000-word article | p95 ≤ 10 ms CPU in optimized build |
| 25,000-word paid-tier text | p95 ≤ 50 ms CPU |
| 10-text batch (500 words each) | p95 ≤ 50 ms CPU |
| 128 KB pathological input | no panic, bounded runtime |
| `metrics: ["coleman_liau"]` vs `metrics: ["all"]` | ≥ 30% faster (validates lazy computation) |

CI should fail on:

- > 10% benchmark regression after stabilized baseline,
- gzip bundle over configured target,
- unapproved dependency growth,
- failed WASM build,
- surviving mutants in formula code.

---

## 20. CI/CD Requirements

### 20.1 CI Steps

1. `cargo fmt --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `cargo test --target wasm32-unknown-unknown` where practical
5. `cargo bench --no-run`
6. `cargo mutants --package readability-core -- --lib formulas` (mutation testing)
7. `cargo insta test` (snapshot regression)
8. `cargo audit` or equivalent security scan
9. License scan for dependencies and word lists
10. Golden formula tests
11. Cross-library comparison report
12. Worker build: `worker-build --release`
13. Bundle size check
14. No-text-logging sentinel test
15. Wrangler dry-run deploy

### 20.2 Environments

| Environment | Source | Purpose |
|-------------|--------|---------|
| Local | working tree | Developer testing via CLI. |
| Preview | every PR | Contract and smoke tests. |
| Staging | `main` | Production-like testing. |
| Production | tagged release | Stable API. |

### 20.3 Rollback

Because the Worker is stateless and the API is versioned, rollback should be safe. Keep previous Worker deployments available through Cloudflare rollback/version tooling.

---

## 21. Documentation Requirements

README must include:

1. What the API does.
2. Quick start (including csvkey setup).
3. Local development instructions.
4. Cloudflare deployment instructions (including `wrangler secret put CSVKEY`).
5. Request/response examples for all three response formats.
6. Batch endpoint usage.
7. Supported metrics and formulas.
8. Warning codes.
9. Score interpretations.
10. Why scores differ from other tools.
11. Tokenizer and syllable-count caveats.
12. Word-list provenance.
13. Privacy and logging policy.
14. Known limitations.
15. Versioning policy.
16. ETag caching behavior.

**Required disclaimer:**

```
Readability formulas are estimates. They do not measure factual accuracy,
tone, audience fit, inclusivity, persuasion, legal sufficiency, medical safety,
or comprehension by a specific reader.
```

---

## 22. Roadmap

### v1.0: Production-ready core Worker

- All canonical readability metrics (Flesch, Flesch-Kincaid, Gunning Fog, Coleman-Liau, SMOG, ARI, Dale-Chall, Spache).
- Additional metrics: TTR, hapax percentage, reading time.
- Plain, HTML, and Markdown-lite input with auto-detection.
- Tiered lazy stats computation.
- Batch endpoint (up to 10 texts).
- Paragraph-level breakdown.
- Hardest-sentences extraction.
- ETag caching support.
- Compact and scores_only response formats.
- Cross-request syllable LRU cache.
- Per-formula error isolation.
- CPU budget with partial results.
- Consensus primary grade with pinned algorithm.
- Privacy-preserving logs.
- csvkey authentication.
- Golden tests, snapshot tests, mutation tests, and cross-library validation.
- CLI for development and corpus management.
- Worker deployment.
- OpenAPI spec (generated from `readability-types`).

### v1.1: Explainability and Diagnostics

- Per-sentence breakdown endpoint.
- Difficult-word samples with privacy controls.
- Sentence-level drivers.
- Optional Linsear Write.
- Optional LIX/RIX.
- Word frequency histogram in debug mode.
- Optional CMU syllable feature if license and size gates pass.
- Grade-level localization (UK key stages, CEFR).

### v1.2: Product Integrations

- Diff endpoint: compare two versions of a text (score deltas).
- TypeScript client SDK (generated from `readability-types` via `ts-rs`).
- Worker service-binding examples.

### v2.0: Internationalization and Long Documents

- Spanish formulas.
- German Flesch.
- Language detection.
- Streaming/paragraph-by-paragraph analysis for long documents.

---

## 23. Risk Register

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Formula mismatch with user expectations | High | Return formula versions; explain canonical definitions. |
| Syllable inaccuracies | High | Exceptions, tests, LRU cache, optional dictionary feature, warnings. |
| Sentence segmentation errors | Medium | Abbreviation rules, tokenizer tests, ambiguity warnings. |
| Word-list licensing issue | High | License review before bundling; provenance docs. |
| GPL contamination | High | Do not copy GPL code/data unless license strategy permits. |
| Worker bundle too large | Medium | Size gate, minimal deps, tunable CMU feature, build-time generation. |
| CPU overrun on large inputs | High | Product-specific body limits, CPU budget with partial results. |
| Raw text logging | High | Logging policy, sentinel tests, code review gate. |
| Short text overconfidence | Medium | Confidence levels and warnings. |
| Non-English input | Medium | Detect/flag unsupported language. |
| API breaking changes | Medium | Versioned endpoints and schema. |
| Cross-library score disagreements | Medium | Explain differences via counts, formulas, tokenizer versions. |
| Degenerate input causing formula panic | Medium | Per-formula Result return type; catch_unwind at boundary. |
| csvkey leak in logs | Medium | Explicit log sanitization; never log query params. |

---

## 24. Acceptance Criteria

### 24.1 Functional

The Worker is acceptable when:

1. `POST /v1/analyze?csvkey=<valid>` returns all supported metrics by default.
2. Missing/invalid csvkey returns 401.
3. Specific metric subsets are supported and trigger only necessary computation tiers.
4. All formulas match fixed-count golden tests against canonical academic definitions.
5. Raw scores are not clamped by default.
6. Optional normalized display values are clearly labeled.
7. Response includes raw statistics (including TTR, hapax, reading time).
8. Response includes engine metadata.
9. Response includes warning codes and confidence.
10. SMOG warns below 30 sentences.
11. Spache warns outside early-grade scope.
12. Dale-Chall returns both raw unadjusted and adjusted score.
13. Coleman-Liau uses letters and sentences per 100 words.
14. ARI character-count policy is explicit.
15. HTML input is safely converted to visible text.
16. Empty, malformed, and oversized inputs are handled safely.
17. Batch endpoint processes up to 10 texts with per-text error isolation.
18. Paragraph breakdown returns per-paragraph difficulty.
19. Hardest-sentences extraction identifies the N most complex sentences.
20. ETag match returns 304 without re-running analysis.
21. All three response formats (`full`, `compact`, `scores_only`) work correctly.
22. A failed formula does not crash the entire analysis.

### 24.2 Non-Functional

The Worker is acceptable when:

1. It compiles to `wasm32-unknown-unknown`.
2. It deploys through Wrangler.
3. It does not log raw submitted text or csvkey.
4. It rejects oversized payloads before body parsing.
5. It handles fuzz inputs without panics.
6. It has unit tests for every formula.
7. It has tokenizer, syllable, word-list, snapshot, and API contract tests.
8. It has mutation tests for formula code with zero surviving mutants.
9. It has bundle-size gates.
10. It has documented formula, tokenizer, syllable, and word-list versions.
11. It has a rollback-safe deployment flow.
12. Lazy computation provides measurable speedup for partial metric requests.
13. Cross-request syllable cache achieves ≥70% hit rate on benchmark corpus.
14. CPU budget prevents any single request from exceeding 50ms.

---

## 25. Implementation Plan

### Phase 1: Types, Schema, and Licensing Lock

**Deliverables:**

1. `readability-types` crate with all request/response structs.
2. Final formula table with canonical academic references.
3. Final JSON schema (derived from types).
4. OpenAPI spec generation.
5. Warning-code registry.
6. Word-list source and license decision.
7. Fixture corpus plan.
8. Cloudflare limits/config decision.

**Key decisions:**

```
formulas = canonical academic definitions only
language = en-US only
raw scores = default
normalization = optional
primary score = consensus_grade (pinned algorithm)
auth = csvkey query param
response formats = full, compact, scores_only
```

### Phase 2: Core Rust Engine

**Deliverables:**

1. `readability-core` crate with tiered computation.
2. Preprocessing modules with auto-detection.
3. Tokenizer with arena allocation.
4. Syllable counter with LRU cache.
5. Dale-Chall and Spache lists via `build.rs` codegen.
6. All formula functions (returning `Result`).
7. Consensus algorithm (pinned math).
8. Paragraph-level breakdown.
9. Hardest-sentences extraction.
10. Warnings/confidence engine.
11. Unit tests + snapshot tests.

### Phase 3: Worker Wrapper

**Deliverables:**

1. `readability-worker` crate.
2. csvkey validation (constant-time, mirrors rusty-gateway).
3. Routes (single + batch + metrics + version + health).
4. Request validation and size limit enforcement.
5. ETag computation and 304 short-circuit.
6. Response format selection.
7. CORS.
8. Error mapping with request_id.
9. Safe logging.
10. Panic recovery boundary.
11. Wrangler config.

### Phase 4: CLI and Validation Harness

**Deliverables:**

1. `readability-cli` crate (analyze, corpus-report, golden-update commands).
2. Fixed-count formula golden tests.
3. Snapshot tests for all fixtures.
4. Cross-library comparison script.
5. Fixture corpus report.
6. Syllable accuracy report.
7. Tokenizer edge-case report.
8. Mutation test baseline.

### Phase 5: Production Hardening

**Deliverables:**

1. CPU budget implementation with partial results.
2. Cross-request syllable cache tuning.
3. Arena allocator benchmarking.
4. Observability structured logs.
5. No-text-logging sentinel test.
6. Size/performance benchmarks.
7. README and API docs.
8. Deployment and rollback guide.
9. `wrangler secret put CSVKEY` documented.

---

## 26. Final Product Shape

The final service should feel like this:

> Send text to a fast edge API. Get a clear primary readability result, Flesch ease, estimated U.S. grade level, every individual formula score, underlying counts, vocabulary diversity metrics, paragraph-level difficulty breakdown, and the hardest sentences flagged for rewriting. Batch 10 texts in one request for SEO pipelines. Choose compact output for high-volume use or full output for detailed analysis. The formulas are implemented from canonical academic definitions and thoroughly tested. Submitted text is not stored or logged by default.

---

## Source Notes for Implementers

The PRD was informed by: canonical academic formula definitions (Flesch 1948, Kincaid et al. 1975, Gunning 1952, Coleman & Liau 1975, McLaughlin 1969, Smith & Senter 1967, Dale & Chall 1948/1995, Spache 1953/revised); Brysbaert 2019 meta-analysis (238 WPM reading speed); Cloudflare Workers Rust documentation; Cloudflare Workers limits documentation; textstat; py-readability-metrics; rusty-gateway authentication pattern; and current Rust/Go/TypeScript readability-library practices. Verify all source licenses before copying code, data, or word lists.
