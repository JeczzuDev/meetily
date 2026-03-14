pub mod commands;
pub mod models;
pub mod web_search;

// Re-export Tauri commands (with their generated __cmd__ variants for generate_handler![])
pub use commands::{
    __cmd__delete_smart_note, __cmd__generate_smart_note, __cmd__get_smart_notes,
    __cmd__get_search_api_key_cmd, __cmd__save_search_api_key_cmd,
    __cmd__reassign_smart_notes_meeting,
    delete_smart_note, generate_smart_note, get_smart_notes,
    get_search_api_key_cmd, save_search_api_key_cmd,
    reassign_smart_notes_meeting,
};
