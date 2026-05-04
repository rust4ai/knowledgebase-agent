use std::sync::Arc;

use async_trait::async_trait;
use metalcraft::{
    create_react_agent, AgentState, CompiledGraph, Executor, GraphError,
    Result as McResult, RunOutcome, Tool, ToolRegistry,
};
use rig::client::CompletionClient;
use rig::providers::openai;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db;

type BoxErr = Box<dyn std::error::Error + Send + Sync>;

// ---------------------------------------------------------------------------
// Tools for the RAG agent
// ---------------------------------------------------------------------------

/// Lists all indexed documents in the knowledge base.
struct ListDocumentsTool {
    db: PgPool,
}

#[async_trait]
impl Tool for ListDocumentsTool {
    fn name(&self) -> &str {
        "list_documents"
    }
    fn description(&self) -> &str {
        "List all indexed documents in the knowledge base. Returns document IDs, filenames, page counts, and status."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }
    async fn call(&self, _args: serde_json::Value) -> McResult<serde_json::Value> {
        let docs = db::documents::list_all(&self.db)
            .await
            .map_err(|e| GraphError::Node {
                node: "list_documents".into(),
                message: e.to_string(),
            })?;

        let results: Vec<serde_json::Value> = docs
            .iter()
            .filter(|d| matches!(d.status, crate::models::document::DocStatus::Indexed))
            .map(|d| {
                json!({
                    "id": d.id.to_string(),
                    "filename": d.filename,
                    "page_count": d.page_count,
                })
            })
            .collect();

        Ok(json!({ "documents": results, "count": results.len() }))
    }
}

/// Searches document indexes to find relevant pages for a query.
/// This is the core PageIndex navigation tool — the agent reasons over
/// the tree structure to identify which pages to read.
struct SearchIndexTool {
    db: PgPool,
}

#[async_trait]
impl Tool for SearchIndexTool {
    fn name(&self) -> &str {
        "search_index"
    }
    fn description(&self) -> &str {
        "Search the document index tree to find relevant pages. Provide a query and optionally a document_id to search within a specific document. Returns the document's root index with page summaries, themes, and entity index so you can reason about which pages to read."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query or topic to find"
                },
                "document_id": {
                    "type": "string",
                    "description": "Optional: specific document UUID to search. If omitted, searches all documents."
                }
            },
            "required": ["query"]
        })
    }
    async fn call(&self, args: serde_json::Value) -> McResult<serde_json::Value> {
        let _query = args["query"].as_str().unwrap_or("");
        let doc_id = args["document_id"].as_str();

        let indexes = if let Some(id_str) = doc_id {
            let id = Uuid::parse_str(id_str).map_err(|e| GraphError::Node {
                node: "search_index".into(),
                message: format!("invalid document_id: {e}"),
            })?;
            match db::page_indexes::get_document_index(&self.db, id)
                .await
                .map_err(|e| GraphError::Node {
                    node: "search_index".into(),
                    message: e.to_string(),
                })? {
                Some(idx) => vec![idx],
                None => vec![],
            }
        } else {
            db::page_indexes::get_all_document_indexes(&self.db)
                .await
                .map_err(|e| GraphError::Node {
                    node: "search_index".into(),
                    message: e.to_string(),
                })?
        };

        let results: Vec<serde_json::Value> = indexes
            .into_iter()
            .map(|idx| {
                json!({
                    "document_id": idx.document_id.to_string(),
                    "root_index": idx.root_index,
                })
            })
            .collect();

        Ok(json!({ "indexes": results, "count": results.len() }))
    }
}

/// Reads the full content of a specific page from a document.
struct ReadPageTool {
    db: PgPool,
}

#[async_trait]
impl Tool for ReadPageTool {
    fn name(&self) -> &str {
        "read_page"
    }
    fn description(&self) -> &str {
        "Read the full text content of a specific page from a document. Use after search_index to retrieve pages identified as relevant."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "document_id": {
                    "type": "string",
                    "description": "The document UUID"
                },
                "page_num": {
                    "type": "integer",
                    "description": "The page number to read (1-based)"
                }
            },
            "required": ["document_id", "page_num"]
        })
    }
    async fn call(&self, args: serde_json::Value) -> McResult<serde_json::Value> {
        let doc_id_str = args["document_id"]
            .as_str()
            .ok_or_else(|| GraphError::Node {
                node: "read_page".into(),
                message: "document_id required".into(),
            })?;
        let doc_id = Uuid::parse_str(doc_id_str).map_err(|e| GraphError::Node {
            node: "read_page".into(),
            message: format!("invalid document_id: {e}"),
        })?;
        let page_num = args["page_num"].as_i64().unwrap_or(1) as i32;

        let page = db::page_indexes::get_page(&self.db, doc_id, page_num)
            .await
            .map_err(|e| GraphError::Node {
                node: "read_page".into(),
                message: e.to_string(),
            })?;

        match page {
            Some(p) => Ok(json!({
                "document_id": doc_id.to_string(),
                "page_num": p.page_num,
                "content": p.content,
                "tree_index": p.tree_index,
            })),
            None => Ok(json!({
                "error": format!("page {page_num} not found in document {doc_id}")
            })),
        }
    }
}

// ---------------------------------------------------------------------------
// RAG Agent builder
// ---------------------------------------------------------------------------

const RAG_SYSTEM_PROMPT: &str = r#"You are a knowledgebase assistant that answers questions using indexed documents.

Your retrieval process follows the PageIndex method:
1. First, use `list_documents` to see what documents are available
2. Use `search_index` to examine document tree indexes and identify relevant pages
3. Reason about the index structure: look at themes, page_map, entity_index, and page summaries to decide which pages contain the answer
4. Use `read_page` to retrieve the full content of the most relevant pages
5. Synthesize an answer from the retrieved content

Important:
- Always cite your sources: mention the document filename and page number
- If information spans multiple pages, read all relevant pages before answering
- If no relevant content is found, say so honestly
- Your reasoning path through the index tree should be traceable — explain why you chose to read specific pages
- Be concise but thorough in your answers"#;

pub struct RagAgent {
    graph: Arc<CompiledGraph<AgentState>>,
}

impl RagAgent {
    pub fn new(config: &AppConfig, db: PgPool) -> std::result::Result<Self, BoxErr> {
        let client = openai::Client::new(&config.openai_api_key)?;
        let model = client.completion_model(&config.openai_model);

        let registry = ToolRegistry::new()
            .register(ListDocumentsTool { db: db.clone() })
            .register(SearchIndexTool { db: db.clone() })
            .register(ReadPageTool { db });

        let graph = create_react_agent(model, registry, RAG_SYSTEM_PROMPT)?;

        Ok(Self {
            graph: Arc::new(graph),
        })
    }

    pub async fn query(&self, question: &str) -> std::result::Result<RagResponse, BoxErr> {
        let executor = Executor::new_from_arc(self.graph.clone()).max_steps(15);
        let state = AgentState::new(question);
        let thread_id = format!("query-{}", Uuid::new_v4());

        let outcome = executor.run(state, &thread_id).await?;

        match outcome {
            RunOutcome::Completed(state) => {
                let answer = state
                    .final_answer()
                    .unwrap_or("No answer could be generated.")
                    .to_string();

                // Extract which tools were called for source tracing
                let tools_called = state.tools_called();
                let reasoning_path: Vec<String> = state
                    .turns()
                    .iter()
                    .filter(|t| !t.tool_calls.is_empty())
                    .map(|t| {
                        t.tool_calls
                            .iter()
                            .map(|tc| format!("{}({})", tc.name, tc.args))
                            .collect::<Vec<_>>()
                            .join(" → ")
                    })
                    .collect();

                Ok(RagResponse {
                    answer,
                    reasoning_path,
                    tools_used: tools_called,
                })
            }
            RunOutcome::Interrupted { reason, .. } => Ok(RagResponse {
                answer: format!("Query interrupted: {reason}"),
                reasoning_path: vec![],
                tools_used: vec![],
            }),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct RagResponse {
    pub answer: String,
    pub reasoning_path: Vec<String>,
    pub tools_used: Vec<String>,
}
