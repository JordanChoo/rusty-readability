#!/usr/bin/env bash
set -euo pipefail

TEST_CSVKEY="test-key-integration-12345"
PORT="${PORT:-8787}"
BASE="http://127.0.0.1:$PORT"

passed=0
failed=0
errors=()

# ── helpers ──────────────────────────────────────────────────────────────────

pass() { ((++passed)); printf "  ✓ %s\n" "$1"; }
fail() { ((++failed)); errors+=("$1: $2"); printf "  ✗ %s — %s\n" "$1" "$2"; }

assert_status() {
  local name="$1" expected="$2" actual="$3"
  if [[ "$actual" == "$expected" ]]; then
    pass "$name"
  else
    fail "$name" "expected status $expected, got $actual"
  fi
}

assert_contains() {
  local name="$1" body="$2" needle="$3"
  if echo "$body" | grep -q "$needle"; then
    pass "$name"
  else
    fail "$name" "response missing '$needle'"
  fi
}

assert_not_contains() {
  local name="$1" body="$2" needle="$3"
  if echo "$body" | grep -q "$needle"; then
    fail "$name" "response should not contain '$needle'"
  else
    pass "$name"
  fi
}

assert_header() {
  local name="$1" headers="$2" header_name="$3"
  if echo "$headers" | grep -qi "^$header_name:"; then
    pass "$name"
  else
    fail "$name" "missing header '$header_name'"
  fi
}

curl_post() {
  local path="$1" data="$2"
  shift 2
  curl -s --max-time 10 -w "\n%{http_code}" \
    -X POST "$BASE${path}?csvkey=$TEST_CSVKEY" \
    -H "Content-Type: application/json" \
    -d "$data" "$@"
}

curl_post_no_auth() {
  local path="$1" data="$2"
  curl -s --max-time 10 -w "\n%{http_code}" \
    -X POST "$BASE${path}" \
    -H "Content-Type: application/json" \
    -d "$data"
}

curl_get() {
  local path="$1"
  curl -s --max-time 10 -w "\n%{http_code}" "$BASE${path}?csvkey=$TEST_CSVKEY"
}

curl_get_no_auth() {
  local path="$1"
  curl -s --max-time 10 -w "\n%{http_code}" "$BASE${path}"
}

split_response() {
  local response="$1"
  BODY=$(echo "$response" | sed '$d')
  STATUS=$(echo "$response" | tail -1)
}

# ── wait for server ──────────────────────────────────────────────────────────

wait_for_server() {
  echo "Waiting for wrangler dev on port $PORT..."
  for i in $(seq 1 30); do
    if curl -s -o /dev/null "$BASE/health" 2>/dev/null; then
      echo "Server ready."
      return 0
    fi
    sleep 1
  done
  echo "ERROR: Server did not start within 30 seconds."
  exit 1
}

# ── test suites ──────────────────────────────────────────────────────────────

test_health() {
  echo ""
  echo "─── Health ───"

  split_response "$(curl_get_no_auth "/health")"
  assert_status "GET /health returns 200" "200" "$STATUS"
  assert_contains "health body has status:ok" "$BODY" '"status"'
  assert_contains "health body has engine" "$BODY" '"engine"'
  assert_contains "health body has version" "$BODY" '"version"'
}

test_auth() {
  echo ""
  echo "─── Auth ───"

  split_response "$(curl_post_no_auth "/v1/analyze" '{"text":"hello world"}')"
  assert_status "missing csvkey returns 401" "401" "$STATUS"
  assert_contains "401 body has UNAUTHORIZED" "$BODY" '"UNAUTHORIZED"'

  split_response "$(curl -s --max-time 10 -w "\n%{http_code}" \
    -X POST "$BASE/v1/analyze?csvkey=wrong-key" \
    -H "Content-Type: application/json" \
    -d '{"text":"hello world"}')"
  assert_status "wrong csvkey returns 401" "401" "$STATUS"

  split_response "$(curl_post "/v1/analyze" '{"text":"The cat sat on the mat. The dog ran fast."}')"
  assert_status "valid csvkey returns 200" "200" "$STATUS"
}

test_analyze() {
  echo ""
  echo "─── POST /v1/analyze ───"

  split_response "$(curl_post "/v1/analyze" '{"text":"The quick brown fox jumped over the lazy dog. Reading is fundamental."}')"
  assert_status "valid text returns 200" "200" "$STATUS"
  assert_contains "response has schema_version" "$BODY" '"schema_version"'
  assert_contains "response has scores" "$BODY" '"scores"'
  assert_contains "response has primary" "$BODY" '"primary"'
  assert_contains "response has flesch_reading_ease" "$BODY" '"flesch_reading_ease"'
  assert_contains "response has flesch_kincaid_grade" "$BODY" '"flesch_kincaid_grade"'
  assert_contains "response has gunning_fog" "$BODY" '"gunning_fog"'
  assert_contains "response has coleman_liau" "$BODY" '"coleman_liau"'
  assert_contains "response has automated_readability_index" "$BODY" '"automated_readability_index"'
  assert_contains "response has dale_chall" "$BODY" '"dale_chall"'

  # Empty text
  split_response "$(curl_post "/v1/analyze" '{"text":""}')"
  assert_status "empty text returns 400" "400" "$STATUS"
  assert_contains "empty text error code" "$BODY" '"MISSING_TEXT"'

  # Missing text field
  split_response "$(curl_post "/v1/analyze" '{"nottext":"hello"}')"
  assert_status "missing text field returns 400" "400" "$STATUS"

  # Punctuation-only text (no words) returns 422
  split_response "$(curl_post "/v1/analyze" '{"text":"!!!"}')"
  assert_status "no-word text returns 422" "422" "$STATUS"
  assert_contains "no-word error code" "$BODY" '"EMPTY_TEXT"'

  # Invalid JSON
  split_response "$(curl_post "/v1/analyze" 'not json')"
  assert_status "invalid JSON returns 400" "400" "$STATUS"
  assert_contains "invalid JSON error code" "$BODY" '"INVALID_JSON"'

  # Wrong content type
  split_response "$(curl -s --max-time 10 -w "\n%{http_code}" \
    -X POST "$BASE/v1/analyze?csvkey=$TEST_CSVKEY" \
    -H "Content-Type: text/plain" \
    -d 'hello world')"
  assert_status "wrong content-type returns 415" "415" "$STATUS"
  assert_contains "415 error code" "$BODY" '"UNSUPPORTED_CONTENT_TYPE"'
}

test_analyze_oversized() {
  echo ""
  echo "─── Oversized payload ───"

  # Generate ~300KB payload (exceeds 256KB MAX_BODY_BYTES) via temp file
  local tmpfile
  tmpfile=$(mktemp)
  python3 -c "import json; print(json.dumps({'text': 'word ' * 60000}))" > "$tmpfile"
  split_response "$(curl -s --max-time 10 -w "\n%{http_code}" \
    -X POST "$BASE/v1/analyze?csvkey=$TEST_CSVKEY" \
    -H "Content-Type: application/json" \
    -d @"$tmpfile")"
  rm -f "$tmpfile"
  assert_status "oversized body returns 413" "413" "$STATUS"
  assert_contains "413 error code" "$BODY" '"PAYLOAD_TOO_LARGE"'
}

test_response_formats() {
  echo ""
  echo "─── Response formats ───"

  local text='{"text":"The quick brown fox jumped over the lazy dog. Reading is fundamental.","options":{"response_format":"compact"}}'
  split_response "$(curl_post "/v1/analyze" "$text")"
  assert_status "compact format returns 200" "200" "$STATUS"
  assert_contains "compact has scores" "$BODY" '"scores"'
  assert_contains "compact has primary" "$BODY" '"primary"'
  assert_not_contains "compact omits engine" "$BODY" '"engine"'

  local text_so='{"text":"The quick brown fox jumped over the lazy dog. Reading is fundamental.","options":{"response_format":"scores_only"}}'
  split_response "$(curl_post "/v1/analyze" "$text_so")"
  assert_status "scores_only format returns 200" "200" "$STATUS"
  assert_contains "scores_only has primary" "$BODY" '"primary"'
  assert_not_contains "scores_only omits engine" "$BODY" '"engine"'
  assert_not_contains "scores_only omits stats" "$BODY" '"stats"'
}

test_etag() {
  echo ""
  echo "─── ETag caching ───"

  local data='{"text":"The cat sat on the mat. The dog ran fast."}'

  # First request — get ETag
  local resp_headers
  resp_headers=$(curl -s --max-time 10 -D - -o /dev/null \
    -X POST "$BASE/v1/analyze?csvkey=$TEST_CSVKEY" \
    -H "Content-Type: application/json" \
    -d "$data")
  local etag
  etag=$(echo "$resp_headers" | grep -i '^etag:' | tr -d '\r' | awk '{print $2}')

  if [[ -z "$etag" ]]; then
    fail "ETag present in response" "no ETag header found"
  else
    pass "ETag present in response"

    # Second request with If-None-Match
    local status
    status=$(curl -s --max-time 10 -o /dev/null -w "%{http_code}" \
      -X POST "$BASE/v1/analyze?csvkey=$TEST_CSVKEY" \
      -H "Content-Type: application/json" \
      -H "If-None-Match: $etag" \
      -d "$data")
    assert_status "matching ETag returns 304" "304" "$status"

    # Different text, same If-None-Match — should NOT 304
    local status2
    status2=$(curl -s --max-time 10 -o /dev/null -w "%{http_code}" \
      -X POST "$BASE/v1/analyze?csvkey=$TEST_CSVKEY" \
      -H "Content-Type: application/json" \
      -H "If-None-Match: $etag" \
      -d '{"text":"Different text entirely. This should not match the old ETag."}'
    )
    assert_status "different text with old ETag returns 200" "200" "$status2"
  fi

  # Same text returns same ETag (determinism)
  local etag1 etag2
  etag1=$(curl -s --max-time 10 -D - -o /dev/null \
    -X POST "$BASE/v1/analyze?csvkey=$TEST_CSVKEY" \
    -H "Content-Type: application/json" \
    -d "$data" | grep -i '^etag:' | tr -d '\r' | awk '{print $2}')
  etag2=$(curl -s --max-time 10 -D - -o /dev/null \
    -X POST "$BASE/v1/analyze?csvkey=$TEST_CSVKEY" \
    -H "Content-Type: application/json" \
    -d "$data" | grep -i '^etag:' | tr -d '\r' | awk '{print $2}')
  if [[ "$etag1" == "$etag2" ]]; then
    pass "deterministic ETag for same input"
  else
    fail "deterministic ETag for same input" "etag1=$etag1, etag2=$etag2"
  fi
}

test_cors() {
  echo ""
  echo "─── CORS ───"

  # Preflight (no auth required)
  local resp_headers
  resp_headers=$(curl -s --max-time 10 -D - -o /dev/null -X OPTIONS "$BASE/v1/analyze")
  local status
  status=$(echo "$resp_headers" | head -1 | grep -o '[0-9]\{3\}')
  assert_status "OPTIONS returns 204" "204" "$status"
  assert_header "preflight has Allow-Origin" "$resp_headers" "access-control-allow-origin"
  assert_header "preflight has Allow-Methods" "$resp_headers" "access-control-allow-methods"
  assert_header "preflight has Allow-Headers" "$resp_headers" "access-control-allow-headers"

  # Normal response includes CORS header
  resp_headers=$(curl -s --max-time 10 -D - -o /dev/null \
    -X POST "$BASE/v1/analyze?csvkey=$TEST_CSVKEY" \
    -H "Content-Type: application/json" \
    -d '{"text":"The cat sat on the mat. The dog ran fast."}')
  assert_header "POST response has Allow-Origin" "$resp_headers" "access-control-allow-origin"
}

test_batch() {
  echo ""
  echo "─── POST /v1/batch ───"

  local data='{"texts":[{"id":"t1","text":"The cat sat on the mat. The dog ran fast."},{"id":"t2","text":"Simple words are good. Kids like short text."}]}'
  split_response "$(curl_post "/v1/batch" "$data")"
  assert_status "batch returns 200" "200" "$STATUS"
  assert_contains "batch has results" "$BODY" '"results"'
  assert_contains "batch has schema_version" "$BODY" '"schema_version"'
  assert_contains "batch result has id t1" "$BODY" '"t1"'
  assert_contains "batch result has id t2" "$BODY" '"t2"'

  # Mixed valid/invalid
  local mixed='{"texts":[{"id":"valid","text":"The cat sat on the mat. The dog ran fast."},{"id":"empty","text":""},{"id":"valid2","text":"Simple words are easy to read for everyone."}]}'
  split_response "$(curl_post "/v1/batch" "$mixed")"
  assert_status "mixed batch returns 200" "200" "$STATUS"
  assert_contains "mixed batch has error status" "$BODY" '"error"'

  # Too many texts (>10)
  local too_many='{"texts":['
  for i in $(seq 1 11); do
    [[ $i -gt 1 ]] && too_many+=","
    too_many+="{\"id\":\"t$i\",\"text\":\"Hello world sentence number $i.\"}"
  done
  too_many+=']}'
  split_response "$(curl_post "/v1/batch" "$too_many")"
  assert_status "too many texts returns 400" "400" "$STATUS"
  assert_contains "too many texts error code" "$BODY" '"BATCH_TOO_MANY_TEXTS"'
}

test_openapi() {
  echo ""
  echo "─── GET /v1/openapi.json ───"

  split_response "$(curl_get_no_auth "/v1/openapi.json")"
  assert_status "openapi returns 200" "200" "$STATUS"
  assert_contains "openapi has openapi field" "$BODY" '"openapi"'
  assert_contains "openapi has paths" "$BODY" '"paths"'
}

test_metrics_and_version() {
  echo ""
  echo "─── GET /v1/metrics & /v1/version ───"

  split_response "$(curl_get "/v1/metrics")"
  assert_status "metrics returns 200" "200" "$STATUS"
  assert_contains "metrics lists flesch" "$BODY" '"flesch_reading_ease"'

  split_response "$(curl_get "/v1/version")"
  assert_status "version returns 200" "200" "$STATUS"
  assert_contains "version has version" "$BODY" '"version"'
  assert_contains "version has engine" "$BODY" '"engine"'
}

test_method_errors() {
  echo ""
  echo "─── Method errors ───"

  split_response "$(curl_get "/v1/analyze")"
  assert_status "GET /v1/analyze returns 404" "404" "$STATUS"

  split_response "$(curl -s --max-time 10 -w "\n%{http_code}" -X PUT "$BASE/v1/analyze?csvkey=$TEST_CSVKEY")"
  assert_status "PUT /v1/analyze returns 405" "405" "$STATUS"
}

test_not_found() {
  echo ""
  echo "─── Not found ───"

  split_response "$(curl_get "/v1/nonexistent")"
  assert_status "unknown path returns 404" "404" "$STATUS"
  assert_contains "404 body has NOT_FOUND" "$BODY" '"NOT_FOUND"'
}

test_html_input() {
  echo ""
  echo "─── HTML input detection ───"

  local data='{"text":"<html><body><p>The cat sat on the mat.</p><p>The dog ran fast.</p><script>evil()</script></body></html>"}'
  split_response "$(curl_post "/v1/analyze" "$data")"
  assert_status "HTML input returns 200" "200" "$STATUS"
  assert_contains "detected html format" "$BODY" '"html"'
  assert_not_contains "script content not in analysis" "$BODY" '"evil"'
}

test_no_nan_infinity() {
  echo ""
  echo "─── No NaN/Infinity in JSON ───"

  local data='{"text":"A single word"}'
  split_response "$(curl_post "/v1/analyze" "$data")"
  assert_not_contains "no NaN in response" "$BODY" "NaN"
  assert_not_contains "no Infinity in response" "$BODY" "Infinity"
}

# ── main ─────────────────────────────────────────────────────────────────────

main() {
  echo "═══════════════════════════════════════════════════════"
  echo "  Worker Integration Tests"
  echo "  Base URL: $BASE"
  echo "═══════════════════════════════════════════════════════"

  wait_for_server

  test_health
  test_auth
  test_analyze
  test_analyze_oversized
  test_response_formats
  test_etag
  test_cors
  test_batch
  test_openapi
  test_metrics_and_version
  test_method_errors
  test_not_found
  test_html_input
  test_no_nan_infinity

  echo ""
  echo "═══════════════════════════════════════════════════════"
  echo "  Results: $passed passed, $failed failed"
  echo "═══════════════════════════════════════════════════════"

  if [[ $failed -gt 0 ]]; then
    echo ""
    echo "Failures:"
    for err in "${errors[@]}"; do
      echo "  ✗ $err"
    done
    exit 1
  fi
}

main "$@"
