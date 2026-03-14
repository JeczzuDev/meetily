use crate::database::repositories::setting::SettingsRepository;
use crate::database::repositories::smart_notes::SmartNotesRepository;
use crate::smart_notes::models::{
    ContextSegment, SearchResult, SmartNote, SmartNoteRequest, SmartNoteResponse,
};
use crate::smart_notes::web_search;
use crate::state::AppState;
use crate::summary::llm_client::{generate_summary, LLMProvider};
use log::{info as log_info, warn as log_warn};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};
use uuid::Uuid;

/// Generates a Smart Note from transcript segments using LLM (+ optional web search)
#[tauri::command]
pub async fn generate_smart_note<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    request: SmartNoteRequest,
) -> Result<SmartNoteResponse, String> {
    log_info!(
        "generate_smart_note called for meeting_id: {}, segment_id: {}",
        request.meeting_id,
        request.segment_id
    );

    let pool = state.db_manager.pool();

    // Resolve app_data_dir (needed for BuiltInAI provider)
    let app_data_dir: Option<PathBuf> = _app.path().app_data_dir().ok();

    // 1. Get model config (provider + model name)
    let config = SettingsRepository::get_model_config(pool)
        .await
        .map_err(|e| format!("Failed to read model config: {}", e))?
        .ok_or_else(|| "No LLM model configured. Please configure a model in Settings.".to_string())?;

    let provider = LLMProvider::from_str(&config.provider)?;
    let model_name = config.model.clone();

    // 2. Get API key for the LLM provider
    let api_key = if provider == LLMProvider::Ollama || provider == LLMProvider::BuiltInAI {
        String::new()
    } else if provider == LLMProvider::CustomOpenAI {
        SettingsRepository::get_custom_openai_config(pool)
            .await
            .map_err(|e| format!("Failed to read custom OpenAI config: {}", e))?
            .and_then(|c| c.api_key)
            .unwrap_or_default()
    } else {
        SettingsRepository::get_api_key(pool, &config.provider)
            .await
            .map_err(|e| format!("Failed to read API key: {}", e))?
            .ok_or_else(|| format!("API key not configured for provider '{}'", config.provider))?
    };

    // 3. Get provider-specific config (Ollama endpoint, CustomOpenAI endpoint)
    let ollama_endpoint = if provider == LLMProvider::Ollama {
        config.ollama_endpoint.clone()
    } else {
        None
    };

    let (custom_openai_endpoint, custom_openai_max_tokens, custom_openai_temperature, custom_openai_top_p) =
        if provider == LLMProvider::CustomOpenAI {
            match SettingsRepository::get_custom_openai_config(pool).await {
                Ok(Some(c)) => (Some(c.endpoint), c.max_tokens.map(|t| t as u32), c.temperature, c.top_p),
                _ => (None, None, None, None),
            }
        } else {
            (None, None, None, None)
        };

    // 4. Optional: Web search
    let search_results = if request.use_web_search {
        match get_search_api_key(pool).await {
            Some(search_key) => {
                let client = reqwest::Client::new();
                // Generate search query using LLM
                let search_query = generate_search_query(
                    &client, &provider, &model_name, &api_key,
                    &request.segment_text, &request.context_segments,
                    ollama_endpoint.as_deref(),
                    custom_openai_endpoint.as_deref(),
                    custom_openai_max_tokens, custom_openai_temperature, custom_openai_top_p,
                    app_data_dir.as_ref(),
                    &request.language,
                ).await;

                match search_query {
                    Ok(query) => {
                        match web_search::search_tavily(&client, &query, &search_key).await {
                            Ok(results) => Some(results),
                            Err(e) => {
                                log_warn!("Web search failed, continuing without sources: {}", e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        log_warn!("Failed to generate search query: {}", e);
                        None
                    }
                }
            }
            None => {
                log_info!("No search API key configured, skipping web search");
                None
            }
        }
    } else {
        None
    };

    // 5. Generate Smart Note content with LLM
    let client = reqwest::Client::new();
    let system_prompt = build_system_prompt(&request.language);
    let user_prompt = build_user_prompt(&request, &search_results);

    let content = generate_summary(
        &client,
        &provider,
        &model_name,
        &api_key,
        &system_prompt,
        &user_prompt,
        ollama_endpoint.as_deref(),
        custom_openai_endpoint.as_deref(),
        custom_openai_max_tokens,
        custom_openai_temperature,
        custom_openai_top_p,
        app_data_dir.as_ref(),
        None, // cancellation_token
    )
    .await
    .map_err(|e| format!("LLM generation failed: {}", e))?;

    // 6. Save to database
    let note_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Ensure meeting row exists (live recordings use a temporary ID not yet in DB)
    sqlx::query(
        "INSERT OR IGNORE INTO meetings (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&request.meeting_id)
    .bind("Live Recording")
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to ensure meeting exists: {}", e))?;

    let sources_json = search_results
        .as_ref()
        .map(|results| serde_json::to_string(results).unwrap_or_default());

    let note = SmartNote {
        id: note_id.clone(),
        meeting_id: request.meeting_id.clone(),
        segment_id: request.segment_id.clone(),
        segment_text: request.segment_text.clone(),
        content: content.clone(),
        sources: sources_json,
        use_web_search: if request.use_web_search { 1 } else { 0 },
        provider: config.provider.clone(),
        model: model_name.clone(),
        created_at: now.clone(),
    };

    SmartNotesRepository::create(pool, &note)
        .await
        .map_err(|e| format!("Failed to save Smart Note: {}", e))?;

    log_info!("Smart Note created: id={}, meeting_id={}", note_id, request.meeting_id);

    // 7. Return response
    Ok(SmartNoteResponse {
        id: note_id,
        meeting_id: request.meeting_id,
        segment_id: request.segment_id,
        content,
        sources: search_results,
        provider: config.provider,
        model: model_name,
        created_at: now,
    })
}

/// Gets all Smart Notes for a meeting
#[tauri::command]
pub async fn get_smart_notes<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    meeting_id: String,
) -> Result<Vec<SmartNoteResponse>, String> {
    let pool = state.db_manager.pool();

    let notes = SmartNotesRepository::get_by_meeting(pool, &meeting_id)
        .await
        .map_err(|e| format!("Failed to get Smart Notes: {}", e))?;

    let responses: Vec<SmartNoteResponse> = notes
        .into_iter()
        .map(|note| {
            let sources: Option<Vec<SearchResult>> = note
                .sources
                .as_ref()
                .and_then(|json| serde_json::from_str(json).ok());

            SmartNoteResponse {
                id: note.id,
                meeting_id: note.meeting_id,
                segment_id: note.segment_id,
                content: note.content,
                sources,
                provider: note.provider,
                model: note.model,
                created_at: note.created_at,
            }
        })
        .collect();

    Ok(responses)
}

/// Deletes a Smart Note by ID
#[tauri::command]
pub async fn delete_smart_note<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    note_id: String,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();

    let deleted = SmartNotesRepository::delete(pool, &note_id)
        .await
        .map_err(|e| format!("Failed to delete Smart Note: {}", e))?;

    if deleted {
        log_info!("Smart Note deleted: id={}", note_id);
        Ok(serde_json::json!({ "message": "Smart Note deleted successfully" }))
    } else {
        Err(format!("Smart Note not found: {}", note_id))
    }
}

/// Returns the stored search API key (empty string if not set)
#[tauri::command]
pub async fn get_search_api_key_cmd<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let pool = state.db_manager.pool();
    Ok(get_search_api_key(pool).await.unwrap_or_default())
}

/// Saves (or clears) the search API key (Tavily)
#[tauri::command]
pub async fn save_search_api_key_cmd<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    api_key: String,
) -> Result<serde_json::Value, String> {
    let pool = state.db_manager.pool();

    // Reuses the braveApiKey column for backward compat — stores Tavily key
    sqlx::query(
        r#"
        INSERT INTO settings (id, provider, model, whisperModel, braveApiKey)
        VALUES ('1', 'openai', 'gpt-4o-2024-11-20', 'large-v3', $1)
        ON CONFLICT(id) DO UPDATE SET braveApiKey = $1
        "#,
    )
    .bind(&api_key)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to save search API key: {}", e))?;

    log_info!("Search API key (Tavily) saved");
    Ok(serde_json::json!({ "status": "success" }))
}

/// Reassigns all Smart Notes from one meeting ID to another.
/// Used when transitioning from a temporary live-recording ID to the final saved meeting ID.
#[tauri::command]
pub async fn reassign_smart_notes_meeting<R: Runtime>(
    _app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    old_meeting_id: String,
    new_meeting_id: String,
) -> Result<u64, String> {
    let pool = state.db_manager.pool();

    let rows = SmartNotesRepository::update_meeting_id(pool, &old_meeting_id, &new_meeting_id)
        .await
        .map_err(|e| format!("Failed to reassign smart notes: {}", e))?;

    log_info!(
        "Reassigned {} smart note(s) from {} to {}",
        rows,
        old_meeting_id,
        new_meeting_id
    );
    Ok(rows)
}

// ── Helper functions ────────────────────────────────────────────

/// Reads search API key from settings table (stored in braveApiKey column)
async fn get_search_api_key(pool: &sqlx::SqlitePool) -> Option<String> {
    let result: Option<String> = sqlx::query_scalar(
        "SELECT braveApiKey FROM settings WHERE id = '1' LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    result.filter(|key| !key.is_empty())
}

/// Uses LLM to generate an optimal search query from the transcript context
async fn generate_search_query(
    client: &reqwest::Client,
    provider: &LLMProvider,
    model_name: &str,
    api_key: &str,
    segment_text: &str,
    context_segments: &[ContextSegment],
    ollama_endpoint: Option<&str>,
    custom_openai_endpoint: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    app_data_dir: Option<&PathBuf>,
    language: &Option<String>,
) -> Result<String, String> {
    let context = context_segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // Resolve language name for search query
    let lang_hint = language
        .as_ref()
        .and_then(|c| language_display_name(c));

    let system = if let Some(name) = lang_hint {
        format!(
            "You are a search query generator. Given a transcript excerpt, \
             output a single concise web search query (max 10 words) in {} that would \
             find relevant information about the main topic discussed. \
             Output ONLY the search query, nothing else.",
            name
        )
    } else {
        "You are a search query generator. Given a transcript excerpt, \
         output a single concise web search query (max 10 words) that would \
         find relevant information about the main topic discussed. \
         Output ONLY the search query, nothing else.".to_string()
    };

    let user = format!(
        "Transcript context:\n{}\n\nHighlighted segment:\n{}",
        context, segment_text
    );

    generate_summary(
        client,
        provider,
        model_name,
        api_key,
        &system,
        &user,
        ollama_endpoint,
        custom_openai_endpoint,
        max_tokens,
        temperature,
        top_p,
        app_data_dir,
        None,
    )
    .await
}

/// Maps a language code to a human-readable name for LLM prompts
fn language_display_name(code: &str) -> Option<&'static str> {
    match code {
        "en" => Some("English"),
        "es" => Some("Spanish"),
        "fr" => Some("French"),
        "de" => Some("German"),
        "it" => Some("Italian"),
        "pt" => Some("Portuguese"),
        "zh" => Some("Chinese"),
        "ja" => Some("Japanese"),
        "ko" => Some("Korean"),
        "ru" => Some("Russian"),
        "ar" => Some("Arabic"),
        "hi" => Some("Hindi"),
        "nl" => Some("Dutch"),
        "sv" => Some("Swedish"),
        "pl" => Some("Polish"),
        "auto" | "" => None,
        _ => None,
    }
}

fn build_system_prompt(language: &Option<String>) -> String {
    let lang_instruction = match language.as_ref().and_then(|c| language_display_name(c)) {
        Some(name) => format!("\n- IMPORTANT: Write the entire note in {}", name),
        None => "\n- IMPORTANT: Write the entire note in the same language as the transcript content".to_string(),
    };

    format!(
        "You are a meeting note assistant. \
         Generate a brief, focused note about the highlighted transcript segment.\
         \n\nRules:\
         \n- Be extremely concise: 2-4 bullet points max\
         \n- Use bold for key terms only\
         \n- Start with a **bold title** (max 6 words)\
         \n- If web search results exist, add one short line citing the most relevant fact\
         \n- Keep the note under 80 words total\
         \n- No filler, no introductions, no conclusions{}",
        lang_instruction
    )
}

fn build_user_prompt(
    request: &SmartNoteRequest,
    search_results: &Option<Vec<SearchResult>>,
) -> String {
    let mut prompt = String::new();

    // Context segments
    prompt.push_str("## Transcript Context\n\n");
    for seg in &request.context_segments {
        prompt.push_str(&format!("[{:.1}s] {}\n", seg.timestamp, seg.text));
    }

    // Highlighted segment
    prompt.push_str(&format!(
        "\n## Highlighted Segment\n\n>>> {}\n",
        request.segment_text
    ));

    // Web search results (if available)
    if let Some(results) = search_results {
        if !results.is_empty() {
            prompt.push_str("\n## Web Search Results\n\n");
            for (i, result) in results.iter().enumerate() {
                prompt.push_str(&format!(
                    "{}. **{}** ({})\n   {}\n\n",
                    i + 1,
                    result.title,
                    result.url,
                    result.snippet
                ));
            }
        }
    }

    prompt.push_str("\nGenerate an informative note about the highlighted topic.");
    prompt
}
