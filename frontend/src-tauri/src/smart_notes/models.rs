use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Database model for a Smart Note (maps 1:1 to smart_notes table)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SmartNote {
    pub id: String,
    pub meeting_id: String,
    pub segment_id: String,
    pub segment_text: String,
    pub content: String,
    pub sources: Option<String>,       // JSON array of SearchResult
    pub use_web_search: i32,           // SQLite boolean: 0 or 1
    pub provider: String,
    pub model: String,
    pub created_at: String,            // ISO 8601 TEXT (matches existing pattern)
}

/// Input from frontend: invoke('generate_smart_note', { request })
#[derive(Debug, Deserialize)]
pub struct SmartNoteRequest {
    pub meeting_id: String,
    pub segment_id: String,
    pub segment_text: String,
    pub context_segments: Vec<ContextSegment>,
    pub use_web_search: bool,
    /// Target language code ("en", "es", …) or "auto" / None for auto-detect
    pub language: Option<String>,
}

/// A transcript segment sent as context (the clicked one + N previous)
#[derive(Debug, Deserialize)]
pub struct ContextSegment {
    pub id: String,
    pub text: String,
    pub timestamp: f64,
}

/// A web search result from Brave Search API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Response sent back to frontend after generating a Smart Note
#[derive(Debug, Serialize)]
pub struct SmartNoteResponse {
    pub id: String,
    pub meeting_id: String,
    pub segment_id: String,
    pub content: String,
    pub sources: Option<Vec<SearchResult>>,
    pub provider: String,
    pub model: String,
    pub created_at: String,
}
