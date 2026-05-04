//! Full integration test: upload Steelhead docs → wait for indexing → query → validate
//!
//! Requires a running server with all services (Postgres, S3, OpenAI).
//!
//! Run with:
//!   cargo test --test integration_test -- --ignored --nocapture
//!
//! Or via dev.sh:
//!   ./dev.sh test --test integration_test -- --ignored --nocapture
//!
//! Environment:
//!   API_URL (default: http://localhost:3000)

use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;

fn api_url() -> String {
    std::env::var("API_URL").unwrap_or_else(|_| {
        let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
        format!("http://localhost:{port}")
    })
}

fn docs_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("docs").join("steelhead")
}

/// Upload a single file, return document ID
async fn upload_doc(client: &reqwest::Client, path: &std::path::Path) -> String {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let bytes = std::fs::read(path).expect("read file");

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(filename.clone())
        .mime_str("text/markdown")
        .unwrap();
    let form = reqwest::multipart::Form::new().part("file", part);

    let resp = client
        .post(format!("{}/api/documents", api_url()))
        .multipart(form)
        .send()
        .await
        .expect("upload request");

    assert_eq!(resp.status(), 201, "upload {filename} should return 201");

    let body: Value = resp.json().await.expect("parse upload response");
    body["id"].as_str().expect("document id").to_string()
}

/// Poll until document reaches "indexed" or "failed" status
async fn wait_for_indexed(client: &reqwest::Client, doc_id: &str, timeout: Duration) -> String {
    let start = std::time::Instant::now();
    loop {
        let resp = client
            .get(format!("{}/api/documents/{doc_id}", api_url()))
            .send()
            .await
            .expect("status request");

        let body: Value = resp.json().await.expect("parse status");
        let status = body["status"]
            .as_str()
            .unwrap_or("unknown")
            .to_lowercase();

        match status.as_str() {
            "indexed" => return status,
            "failed" => {
                let err = body["error_msg"].as_str().unwrap_or("unknown error");
                panic!("Document {doc_id} failed indexing: {err}");
            }
            _ => {
                if start.elapsed() > timeout {
                    panic!(
                        "Timeout waiting for {doc_id} to index ({}s). Last status: {status}",
                        timeout.as_secs()
                    );
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

/// Query the knowledge base and return the answer string
async fn query(client: &reqwest::Client, question: &str) -> Value {
    let resp = client
        .post(format!("{}/api/query", api_url()))
        .json(&serde_json::json!({ "question": question }))
        .timeout(Duration::from_secs(120))
        .send()
        .await
        .expect("query request");

    assert!(resp.status().is_success(), "query should succeed");
    resp.json().await.expect("parse query response")
}

/// Delete a document (cleanup)
async fn delete_doc(client: &reqwest::Client, doc_id: &str) {
    let _ = client
        .delete(format!("{}/api/documents/{doc_id}", api_url()))
        .send()
        .await;
}

/// Assert answer contains substring (case-insensitive)
fn assert_answer_contains(response: &Value, expected: &str, context: &str) {
    let answer = response["answer"]
        .as_str()
        .unwrap_or("")
        .to_lowercase();
    let expected_lower = expected.to_lowercase();
    assert!(
        answer.contains(&expected_lower),
        "{context}: expected answer to contain '{expected}'\nGot: {}",
        &response["answer"].as_str().unwrap_or("")[..300.min(answer.len())]
    );
}

// =============================================================================
// Tests (all #[ignore] — only run explicitly)
// =============================================================================

/// Full end-to-end: upload all 5 docs, index, then query
#[tokio::test]
#[ignore]
async fn test_steelhead_full_pipeline() {
    let client = reqwest::Client::new();

    // Health check
    let health = client
        .get(format!("{}/api/health", api_url()))
        .send()
        .await;
    assert!(
        health.is_ok() && health.unwrap().status().is_success(),
        "Server must be running at {}",
        api_url()
    );

    // Upload all docs
    let docs_path = docs_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&docs_path)
        .expect("read docs/steelhead dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    assert!(
        entries.len() >= 5,
        "Expected at least 5 docs in {}",
        docs_path.display()
    );

    let mut doc_ids = Vec::new();
    for entry in &entries {
        let id = upload_doc(&client, &entry.path()).await;
        eprintln!("Uploaded {} → {id}", entry.file_name().to_string_lossy());
        doc_ids.push(id);
    }

    // Wait for all to be indexed
    eprintln!("Waiting for indexing...");
    let timeout = Duration::from_secs(300);
    for id in &doc_ids {
        let status = wait_for_indexed(&client, id, timeout).await;
        eprintln!("  {id}: {status}");
    }

    // Run queries
    let queries_and_expectations: Vec<(&str, &str, &str)> = vec![
        (
            "How much funding did Steelhead raise from Mainsail Partners?",
            "84",
            "funding_amount",
        ),
        (
            "What AI features does the Steelhead platform offer?",
            "schedul",
            "ai_features",
        ),
        (
            "How long does a Steelhead deployment take?",
            "week",
            "deployment_timeline",
        ),
        (
            "What was new in Steelhead v3.2?",
            "operator",
            "release_notes",
        ),
        (
            "How do I fix a barcode scanner not working in Steelhead?",
            "scan",
            "troubleshooting",
        ),
    ];

    for (question, expected, label) in &queries_and_expectations {
        eprintln!("Query [{label}]: {question}");
        let response = query(&client, question).await;
        assert_answer_contains(&response, expected, label);
        eprintln!("  PASS — answer contains '{expected}'");
    }

    // Cleanup
    eprintln!("Cleaning up...");
    for id in &doc_ids {
        delete_doc(&client, id).await;
    }

    eprintln!("All integration tests passed!");
}

/// Test: upload a single doc and verify it appears in document list
#[tokio::test]
#[ignore]
async fn test_upload_and_list() {
    let client = reqwest::Client::new();
    let doc_path = docs_dir().join("01-company-overview.md");
    let id = upload_doc(&client, &doc_path).await;

    // Verify it shows up in the list
    let resp = client
        .get(format!("{}/api/documents", api_url()))
        .send()
        .await
        .expect("list request");
    let docs: Vec<Value> = resp.json().await.expect("parse list");
    let found = docs.iter().any(|d| d["id"].as_str() == Some(&id));
    assert!(found, "Uploaded doc {id} should appear in list");

    // Cleanup
    delete_doc(&client, &id).await;
}

/// Test: query about cross-document topic (processes + quoting spans docs 1, 2, 3)
#[tokio::test]
#[ignore]
async fn test_cross_document_query() {
    let client = reqwest::Client::new();

    // Upload 2 relevant docs
    let paths = [
        docs_dir().join("01-company-overview.md"),
        docs_dir().join("02-platform-features.md"),
    ];

    let mut ids = Vec::new();
    for p in &paths {
        ids.push(upload_doc(&client, p).await);
    }

    let timeout = Duration::from_secs(300);
    for id in &ids {
        wait_for_indexed(&client, id, timeout).await;
    }

    let response = query(
        &client,
        "What manufacturing processes does Steelhead support and how does the quoting system work?",
    )
    .await;

    // Should mention both processes (from overview) and quoting features
    let answer = response["answer"].as_str().unwrap_or("").to_lowercase();
    assert!(
        answer.contains("plat") || answer.contains("anodiz") || answer.contains("powder"),
        "Answer should mention manufacturing processes"
    );
    assert!(
        answer.contains("quot"),
        "Answer should mention quoting"
    );

    for id in &ids {
        delete_doc(&client, id).await;
    }
}
