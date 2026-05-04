#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# Steelhead Knowledgebase Integration Test
# =============================================================================
# Uploads 5 Steelhead docs, waits for indexing, then runs queries and validates
# that answers contain expected content from the documents.
#
# Prerequisites:
#   - Server running (./dev.sh run)
#   - All env vars set (DATABASE_URL, S3_*, OPENAI_API_KEY)
#
# Usage:
#   ./tests/integration_steelhead.sh              # default: http://localhost:3000
#   API_URL=http://localhost:8080 ./tests/integration_steelhead.sh
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DOCS_DIR="$PROJECT_DIR/docs/steelhead"
API_URL="${API_URL:-http://localhost:${PORT:-3000}}"

PASS=0
FAIL=0
UPLOADED_IDS=()

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()  { echo -e "${YELLOW}[TEST]${NC} $*"; }
pass() { echo -e "${GREEN}[PASS]${NC} $*"; ((PASS++)); }
fail() { echo -e "${RED}[FAIL]${NC} $*"; ((FAIL++)); }

# ---------------------------------------------------------------------------
# Cleanup: delete uploaded docs on exit
# ---------------------------------------------------------------------------
cleanup() {
    log "Cleaning up ${#UPLOADED_IDS[@]} uploaded documents..."
    for id in "${UPLOADED_IDS[@]}"; do
        curl -s -X DELETE "$API_URL/api/documents/$id" > /dev/null 2>&1 || true
    done
    log "Cleanup done."
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Health check
# ---------------------------------------------------------------------------
log "Checking server at $API_URL..."
HEALTH=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/api/health" 2>/dev/null || echo "000")
if [ "$HEALTH" != "200" ]; then
    fail "Server not reachable at $API_URL (HTTP $HEALTH)"
    echo "Start the server first: ./dev.sh run"
    exit 1
fi
pass "Server healthy"

# ---------------------------------------------------------------------------
# Phase 1: Upload all Steelhead docs
# ---------------------------------------------------------------------------
log "Phase 1: Uploading Steelhead documents..."

for doc in "$DOCS_DIR"/*.md; do
    FILENAME=$(basename "$doc")
    log "  Uploading $FILENAME..."

    RESPONSE=$(curl -s -X POST "$API_URL/api/documents" -F "file=@$doc")
    DOC_ID=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null || echo "")

    if [ -z "$DOC_ID" ]; then
        fail "Upload failed for $FILENAME: $RESPONSE"
        continue
    fi

    UPLOADED_IDS+=("$DOC_ID")
    pass "Uploaded $FILENAME → $DOC_ID"
done

if [ ${#UPLOADED_IDS[@]} -eq 0 ]; then
    fail "No documents uploaded. Aborting."
    exit 1
fi

log "Uploaded ${#UPLOADED_IDS[@]} documents"

# ---------------------------------------------------------------------------
# Phase 2: Wait for indexing to complete
# ---------------------------------------------------------------------------
log "Phase 2: Waiting for indexing (polling every 10s, timeout 5min)..."

MAX_WAIT=300
ELAPSED=0
ALL_INDEXED=false

while [ $ELAPSED -lt $MAX_WAIT ]; do
    INDEXED_COUNT=0
    FAILED_COUNT=0

    for id in "${UPLOADED_IDS[@]}"; do
        STATUS=$(curl -s "$API_URL/api/documents/$id" | \
            python3 -c "import sys,json; print(json.load(sys.stdin)['status'])" 2>/dev/null || echo "unknown")

        case "$STATUS" in
            indexed|Indexed)  ((INDEXED_COUNT++)) ;;
            failed|Failed)    ((FAILED_COUNT++))  ;;
        esac
    done

    log "  Progress: $INDEXED_COUNT indexed, $FAILED_COUNT failed, $((${#UPLOADED_IDS[@]} - INDEXED_COUNT - FAILED_COUNT)) pending (${ELAPSED}s)"

    if [ $((INDEXED_COUNT + FAILED_COUNT)) -eq ${#UPLOADED_IDS[@]} ]; then
        ALL_INDEXED=true
        break
    fi

    sleep 10
    ELAPSED=$((ELAPSED + 10))
done

if [ "$ALL_INDEXED" = true ]; then
    pass "All documents finished processing in ${ELAPSED}s ($INDEXED_COUNT indexed, $FAILED_COUNT failed)"
else
    fail "Timeout after ${MAX_WAIT}s — not all documents indexed"
fi

if [ $FAILED_COUNT -gt 0 ]; then
    fail "$FAILED_COUNT documents failed indexing"
fi

# ---------------------------------------------------------------------------
# Phase 3: Query tests
# ---------------------------------------------------------------------------
log "Phase 3: Running query tests..."

# Helper: query and check answer contains expected substring (case-insensitive)
assert_query() {
    local QUESTION="$1"
    local EXPECTED="$2"
    local LABEL="$3"

    log "  Query: $QUESTION"

    RESPONSE=$(curl -s -X POST "$API_URL/api/query" \
        -H "Content-Type: application/json" \
        -d "{\"question\": \"$QUESTION\"}" \
        --max-time 120)

    ANSWER=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('answer',''))" 2>/dev/null || echo "")

    if [ -z "$ANSWER" ]; then
        fail "$LABEL — no answer returned"
        echo "    Response: $RESPONSE"
        return
    fi

    # Case-insensitive check
    if echo "$ANSWER" | grep -qi "$EXPECTED"; then
        pass "$LABEL — answer contains '$EXPECTED'"
    else
        fail "$LABEL — expected '$EXPECTED' in answer"
        echo "    Answer (truncated): ${ANSWER:0:300}"
    fi
}

# --- Test queries covering each document ---

# Doc 1: Company Overview
assert_query \
    "How much funding did Steelhead Technologies raise from Mainsail Partners?" \
    "84" \
    "Funding amount"

# Doc 2: Platform Features
assert_query \
    "What AI features does the Steelhead platform offer?" \
    "schedul" \
    "AI features (smart scheduling)"

# Doc 3: Deployment Guide
assert_query \
    "How long does it take to deploy Steelhead at a job shop?" \
    "week" \
    "Deployment timeline"

# Doc 4: Release Notes
assert_query \
    "What new features were added in Steelhead version 3.2?" \
    "operator" \
    "v3.2 release features"

# Doc 5: Troubleshooting
assert_query \
    "What should I do if the barcode scanner is not working in Steelhead?" \
    "scan" \
    "Scanner troubleshooting"

# Cross-document query
assert_query \
    "What manufacturing processes does Steelhead support and how does quoting work?" \
    "quot" \
    "Cross-doc: processes + quoting"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "============================================="
echo -e "  Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}"
echo "============================================="

if [ $FAIL -gt 0 ]; then
    exit 1
fi
