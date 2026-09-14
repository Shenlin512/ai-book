use crate::db;
use crate::llm;
use crate::models::*;
use crate::prompt;
use crate::seed;
use crate::AppState;
use rusqlite::Connection;
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

fn with_db<T>(state: &AppState, f: impl FnOnce(&Connection) -> Result<T, String>) -> Result<T, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    f(&conn)
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    with_db(&state, db::list_projects)
}

#[tauri::command]
pub fn get_project_bundle(state: State<AppState>, id: String) -> Result<ProjectBundle, String> {
    with_db(&state, |conn| db::load_bundle(conn, &id))
}

#[tauri::command]
pub fn upsert_project(state: State<AppState>, project: Project) -> Result<Project, String> {
    with_db(&state, |conn| db::upsert_project(conn, project))
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> Result<(), String> {
    with_db(&state, |conn| db::delete_project(conn, &id))
}

#[tauri::command]
pub fn upsert_character(state: State<AppState>, item: Character) -> Result<Character, String> {
    with_db(&state, |conn| db::upsert_character(conn, item))
}

#[tauri::command]
pub fn delete_character(state: State<AppState>, id: String) -> Result<(), String> {
    with_db(&state, |conn| db::delete_character(conn, &id))
}

#[tauri::command]
pub fn upsert_entry(state: State<AppState>, item: Entry) -> Result<Entry, String> {
    with_db(&state, |conn| db::upsert_entry(conn, item))
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, id: String) -> Result<(), String> {
    with_db(&state, |conn| db::delete_entry(conn, &id))
}

#[tauri::command]
pub fn upsert_outline(state: State<AppState>, item: Outline) -> Result<Outline, String> {
    with_db(&state, |conn| db::upsert_outline(conn, item))
}

#[tauri::command]
pub fn delete_outline(state: State<AppState>, id: String) -> Result<(), String> {
    with_db(&state, |conn| db::delete_outline(conn, &id))
}

#[tauri::command]
pub fn upsert_chapter(state: State<AppState>, item: Chapter) -> Result<Chapter, String> {
    with_db(&state, |conn| db::upsert_chapter(conn, item))
}

#[tauri::command]
pub fn delete_chapter(state: State<AppState>, id: String) -> Result<(), String> {
    with_db(&state, |conn| db::delete_chapter(conn, &id))
}

#[tauri::command]
pub fn upsert_memory(state: State<AppState>, item: Memory) -> Result<Memory, String> {
    with_db(&state, |conn| db::upsert_memory(conn, item))
}

#[tauri::command]
pub fn delete_memory(state: State<AppState>, id: String) -> Result<(), String> {
    with_db(&state, |conn| db::delete_memory(conn, &id))
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    with_db(&state, db::get_settings)
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: AppSettings) -> Result<AppSettings, String> {
    with_db(&state, |conn| {
        db::save_settings(conn, &settings)?;
        Ok(settings)
    })
}

#[tauri::command]
pub async fn test_model(state: State<'_, AppState>) -> Result<ConnectionTest, String> {
    let settings = with_db(&state, db::get_settings)?;
    llm::test_connection(&settings).await
}

#[tauri::command]
pub fn seed_demo_project(state: State<AppState>) -> Result<Project, String> {
    with_db(&state, seed::seed_demo)
}

#[tauri::command]
pub fn preview_context(state: State<AppState>, request: GenerateRequest) -> Result<ContextPreview, String> {
    with_db(&state, |conn| {
        let bundle = db::load_bundle(conn, &request.project_id)?;
        let settings = db::get_settings(conn)?;
        Ok(prompt::build_context(&bundle, &settings, &request))
    })
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamPayload {
    token: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DonePayload {
    text: String,
    cancelled: bool,
}

#[tauri::command]
pub async fn generate_text(
    app: AppHandle,
    state: State<'_, AppState>,
    request: GenerateRequest,
) -> Result<String, String> {
    state.cancel.store(false, Ordering::SeqCst);
    let (settings, preview) = with_db(&state, |conn| {
        let bundle = db::load_bundle(conn, &request.project_id)?;
        let settings = db::get_settings(conn)?;
        let preview = prompt::build_context(&bundle, &settings, &request);
        Ok((settings, preview))
    })?;

    if settings.base_url.trim().is_empty() || settings.model.trim().is_empty() {
        return Err("请先在设置里配置模型地址和模型名。".into());
    }

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: preview.system_prompt,
        },
        ChatMessage {
            role: "user".into(),
            content: preview.user_prompt,
        },
    ];

    let app_chunk = app.clone();
    let result = llm::chat_stream(&settings, messages, &state.cancel, move |token| {
        app_chunk
            .emit("llm-token", StreamPayload { token: token.to_string() })
            .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(text) => {
            let cancelled = state.cancel.load(Ordering::SeqCst);
            let _ = app.emit(
                "llm-done",
                DonePayload {
                    text: text.clone(),
                    cancelled,
                },
            );
            if request.mode == "memory" && !cancelled {
                let _ = persist_extracted_memories(&state, &request, &text);
            }
            if request.mode == "summarize" && !cancelled {
                let _ = persist_summary(&state, &request, &text);
            }
            Ok(text)
        }
        Err(err) => {
            let _ = app.emit("llm-error", err.clone());
            Err(err)
        }
    }
}

#[tauri::command]
pub async fn generate_book_meta(
    state: State<'_, AppState>,
    seed: BookMetaSeed,
) -> Result<BookMeta, String> {
    let settings = with_db(&state, db::get_settings)?;
    if settings.base_url.trim().is_empty() || settings.model.trim().is_empty() {
        return Err("请先在设置里配置模型地址和模型名。".into());
    }
    let text = llm::chat_once(&settings, prompt::book_meta_messages(&seed)).await?;
    parse_book_meta(&text)
}

fn parse_book_meta(text: &str) -> Result<BookMeta, String> {
    let start = text.find('{').ok_or_else(|| "模型没有返回书稿 JSON。".to_string())?;
    let end = text.rfind('}').ok_or_else(|| "模型没有返回书稿 JSON。".to_string())?;
    if end <= start {
        return Err("模型没有返回书稿 JSON。".into());
    }
    let value: serde_json::Value =
        serde_json::from_str(&text[start..=end]).map_err(|e| format!("书稿 JSON 无效：{e}"))?;
    let pick = |keys: &[&str]| {
        keys.iter()
            .find_map(|key| value.get(*key).and_then(|v| v.as_str()))
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let meta = BookMeta {
        title: pick(&["title", "书名"]),
        genre: pick(&["genre", "体裁", "载体"]),
        style_prompt: pick(&["stylePrompt", "style_prompt", "文风", "文风提示"]),
        description: pick(&["description", "简介", "一句话简介"]),
    };
    if meta.title.is_empty() {
        return Err("模型没有给出书名。".into());
    }
    Ok(meta)
}

#[tauri::command]
pub fn cancel_generate(state: State<AppState>) -> Result<(), String> {
    state.cancel.store(true, Ordering::SeqCst);
    Ok(())
}

fn persist_extracted_memories(
    state: &AppState,
    request: &GenerateRequest,
    text: &str,
) -> Result<(), String> {
    let json = extract_json_array(text).ok_or_else(|| "未能解析记忆 JSON".to_string())?;
    let items: Vec<serde_json::Value> =
        serde_json::from_str(&json).map_err(|e| format!("记忆 JSON 无效：{e}"))?;
    with_db(state, |conn| {
        for item in items {
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("未命名记忆")
                .to_string();
            let content = item
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if content.trim().is_empty() {
                continue;
            }
            let kind = item
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("plot")
                .to_string();
            let source = item
                .get("source")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("历史章节")
                .to_string();
            db::upsert_memory(
                conn,
                Memory {
                    id: String::new(),
                    project_id: request.project_id.clone(),
                    kind,
                    title,
                    content,
                    source,
                    created_at: String::new(),
                    updated_at: String::new(),
                },
            )?;
        }
        Ok(())
    })
}

fn persist_summary(state: &AppState, request: &GenerateRequest, text: &str) -> Result<(), String> {
    let Some(chapter_id) = &request.chapter_id else {
        return Ok(());
    };
    with_db(state, |conn| {
        if let Some(mut chapter) = db::get_chapter(conn, chapter_id)? {
            chapter.summary = text.trim().to_string();
            db::upsert_chapter(conn, chapter)?;
        }
        Ok(())
    })
}

fn extract_json_array(text: &str) -> Option<String> {
    let start = text.find('[')?;
    let end = text.rfind(']')?;
    if end > start {
        Some(text[start..=end].to_string())
    } else {
        None
    }
}
