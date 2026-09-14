mod commands;
mod db;
mod llm;
mod models;
mod prompt;
mod retrieval;
mod seed;

use rusqlite::Connection;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub cancel: AtomicBool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            std::fs::create_dir_all(&dir)?;
            let conn = db::open(&dir.join("mohe.db"))?;
            app.manage(AppState {
                db: Mutex::new(conn),
                cancel: AtomicBool::new(false),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_projects,
            commands::get_project_bundle,
            commands::upsert_project,
            commands::delete_project,
            commands::upsert_character,
            commands::delete_character,
            commands::upsert_entry,
            commands::delete_entry,
            commands::upsert_outline,
            commands::delete_outline,
            commands::upsert_chapter,
            commands::delete_chapter,
            commands::upsert_memory,
            commands::delete_memory,
            commands::get_settings,
            commands::save_settings,
            commands::test_model,
            commands::seed_demo_project,
            commands::preview_context,
            commands::generate_text,
            commands::generate_book_meta,
            commands::cancel_generate,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
