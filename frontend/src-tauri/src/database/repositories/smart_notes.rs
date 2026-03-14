use crate::smart_notes::models::SmartNote;
use sqlx::SqlitePool;

pub struct SmartNotesRepository;

impl SmartNotesRepository {
    /// Insert a new Smart Note into the database
    pub async fn create(
        pool: &SqlitePool,
        note: &SmartNote,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO smart_notes (id, meeting_id, segment_id, segment_text, content, sources, use_web_search, provider, model, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&note.id)
        .bind(&note.meeting_id)
        .bind(&note.segment_id)
        .bind(&note.segment_text)
        .bind(&note.content)
        .bind(&note.sources)
        .bind(note.use_web_search)
        .bind(&note.provider)
        .bind(&note.model)
        .bind(&note.created_at)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Get all Smart Notes for a meeting, ordered by creation time
    pub async fn get_by_meeting(
        pool: &SqlitePool,
        meeting_id: &str,
    ) -> Result<Vec<SmartNote>, sqlx::Error> {
        let notes = sqlx::query_as::<_, SmartNote>(
            "SELECT * FROM smart_notes WHERE meeting_id = $1 ORDER BY created_at ASC",
        )
        .bind(meeting_id)
        .fetch_all(pool)
        .await?;
        Ok(notes)
    }

    /// Get a single Smart Note by ID
    pub async fn get_by_id(
        pool: &SqlitePool,
        note_id: &str,
    ) -> Result<Option<SmartNote>, sqlx::Error> {
        let note = sqlx::query_as::<_, SmartNote>(
            "SELECT * FROM smart_notes WHERE id = $1",
        )
        .bind(note_id)
        .fetch_optional(pool)
        .await?;
        Ok(note)
    }

    /// Delete a Smart Note by ID
    pub async fn delete(
        pool: &SqlitePool,
        note_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM smart_notes WHERE id = $1")
            .bind(note_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Reassign all Smart Notes from one meeting_id to another
    pub async fn update_meeting_id(
        pool: &SqlitePool,
        old_meeting_id: &str,
        new_meeting_id: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE smart_notes SET meeting_id = $1 WHERE meeting_id = $2",
        )
        .bind(new_meeting_id)
        .bind(old_meeting_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}
