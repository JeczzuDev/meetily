/**
 * Smart Notes types — mirrors Rust structs from smart_notes/models.rs
 */

/** A transcript segment sent as context (the clicked one + N previous) */
export interface ContextSegment {
  id: string;
  text: string;
  timestamp: number;
}

/** Input for generate_smart_note command */
export interface SmartNoteRequest {
  meeting_id: string;
  segment_id: string;
  segment_text: string;
  context_segments: ContextSegment[];
  use_web_search: boolean;
  language?: string;
}

/** A web search result from Brave Search API */
export interface SearchResult {
  title: string;
  url: string;
  snippet: string;
}

/** Response from generate_smart_note / get_smart_notes commands */
export interface SmartNoteResponse {
  id: string;
  meeting_id: string;
  segment_id: string;
  content: string;
  sources: SearchResult[] | null;
  provider: string;
  model: string;
  created_at: string;
}

/** Status of the smart note generation pipeline */
export type SmartNoteStatus = 'idle' | 'generating' | 'error';
