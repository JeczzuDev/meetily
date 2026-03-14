-- Smart Notes table for AI-generated contextual notes from transcript segments
CREATE TABLE IF NOT EXISTS smart_notes (
    id TEXT PRIMARY KEY NOT NULL,
    meeting_id TEXT NOT NULL,
    segment_id TEXT NOT NULL,
    segment_text TEXT NOT NULL,
    content TEXT NOT NULL,
    sources TEXT,
    use_web_search INTEGER NOT NULL DEFAULT 0,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE
);

-- Index for fast lookups by meeting
CREATE INDEX IF NOT EXISTS idx_smart_notes_meeting_id ON smart_notes(meeting_id);

-- Add Brave Search API key to settings
ALTER TABLE settings ADD COLUMN braveApiKey TEXT;
