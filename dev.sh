#!/usr/bin/env bash
set -euo pipefail

# knowledgebase-agent dev script
# Usage:
#   ./dev.sh              - run backend + frontend together
#   ./dev.sh run          - run backend only
#   ./dev.sh frontend     - run frontend only
#   ./dev.sh test         - run tests
#   ./dev.sh check        - cargo check
#   ./dev.sh upload FILE  - upload a document via curl
#   ./dev.sh docs         - list all documents
#   ./dev.sh query "..."  - query the knowledge base
#   ./dev.sh status ID    - get document status

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Load .env if present
if [ -f .env ]; then
    set -a
    source .env
    set +a
fi

API_URL="${API_URL:-http://localhost:${PORT:-3000}}"

# Cleanup background processes on exit
cleanup() {
    if [ -n "${BACKEND_PID:-}" ]; then kill "$BACKEND_PID" 2>/dev/null; fi
    if [ -n "${FRONTEND_PID:-}" ]; then kill "$FRONTEND_PID" 2>/dev/null; fi
    wait 2>/dev/null
}

case "${1:-dev}" in
    dev)
        trap cleanup EXIT INT TERM

        echo "Starting backend on :${PORT:-3000}..."
        RUST_LOG="${RUST_LOG:-info}" cargo run &
        BACKEND_PID=$!

        echo "Starting frontend on :5173..."
        cd frontend && npm run dev &
        FRONTEND_PID=$!
        cd "$SCRIPT_DIR"

        echo ""
        echo "  Backend:  http://localhost:${PORT:-3000}"
        echo "  Frontend: http://localhost:5173"
        echo ""

        wait
        ;;

    run)
        echo "Starting knowledgebase-agent on ${API_URL}..."
        RUST_LOG="${RUST_LOG:-info}" cargo run
        ;;

    frontend)
        cd frontend && npm run dev
        ;;

    watch)
        echo "Starting knowledgebase-agent with cargo-watch..."
        RUST_LOG="${RUST_LOG:-info}" cargo watch -x run
        ;;

    test)
        echo "Running tests..."
        RUST_LOG="${RUST_LOG:-info}" cargo test "${@:2}"
        ;;

    check)
        cargo check
        ;;

    upload)
        FILE="${2:?Usage: ./dev.sh upload <filepath>}"
        if [ ! -f "$FILE" ]; then
            echo "File not found: $FILE"
            exit 1
        fi
        echo "Uploading $FILE..."
        curl -s -X POST "$API_URL/api/documents" \
            -F "file=@$FILE" | python3 -m json.tool 2>/dev/null || cat
        echo
        ;;

    docs)
        echo "Listing documents..."
        curl -s "$API_URL/api/documents" | python3 -m json.tool 2>/dev/null || cat
        echo
        ;;

    query)
        QUESTION="${2:?Usage: ./dev.sh query \"your question\"}"
        echo "Querying: $QUESTION"
        curl -s -X POST "$API_URL/api/query" \
            -H "Content-Type: application/json" \
            -d "{\"question\": \"$QUESTION\"}" | python3 -m json.tool 2>/dev/null || cat
        echo
        ;;

    status)
        DOC_ID="${2:?Usage: ./dev.sh status <document-id>}"
        curl -s "$API_URL/api/documents/$DOC_ID" | python3 -m json.tool 2>/dev/null || cat
        echo
        ;;

    health)
        curl -s "$API_URL/api/health"
        echo
        ;;

    *)
        echo "Usage: ./dev.sh [dev|run|frontend|watch|test|check|upload|docs|query|status|health]"
        exit 1
        ;;
esac
