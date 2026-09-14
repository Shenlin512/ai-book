use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub title: String,
    pub genre: String,
    pub description: String,
    pub style_prompt: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub aliases: String,
    pub role: String,
    pub personality: String,
    pub appearance: String,
    pub background: String,
    pub goals: String,
    pub relationships: String,
    pub current_state: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub category: String,
    pub content: String,
    pub tags: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Outline {
    pub id: String,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub content: String,
    pub order_index: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: String,
    pub project_id: String,
    pub outline_id: Option<String>,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub order_index: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub title: String,
    pub content: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBundle {
    pub project: Project,
    pub characters: Vec<Character>,
    pub entries: Vec<Entry>,
    pub outlines: Vec<Outline>,
    pub chapters: Vec<Chapter>,
    pub memories: Vec<Memory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub mode: String,
    pub preset: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub retrieve_k: u32,
    pub context_budget: u32,
    pub include_memories: bool,
    pub include_retrieval: bool,
    pub include_outline: bool,
    pub include_characters: bool,
    pub include_entries: bool,
    pub writing_style: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            mode: "online".into(),
            preset: "deepseek".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            api_key: String::new(),
            model: "deepseek-chat".into(),
            temperature: 0.85,
            max_tokens: 2048,
            retrieve_k: 8,
            context_budget: 7000,
            include_memories: true,
            include_retrieval: true,
            include_outline: true,
            include_characters: true,
            include_entries: true,
            writing_style: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRequest {
    pub project_id: String,
    pub chapter_id: Option<String>,
    pub outline_id: Option<String>,
    #[serde(default)]
    pub character_id: Option<String>,
    #[serde(default)]
    pub entry_id: Option<String>,
    pub character_ids: Vec<String>,
    pub entry_ids: Vec<String>,
    pub instruction: String,
    pub mode: String,
    #[serde(default)]
    pub target: String,
    pub selected_text: Option<String>,
    #[serde(default)]
    pub object_snapshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextItem {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPreview {
    pub items: Vec<ContextItem>,
    pub system_prompt: String,
    pub user_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookMetaSeed {
    pub title: String,
    pub genre: String,
    pub style_prompt: String,
    pub description: String,
    pub instruction: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookMeta {
    pub title: String,
    pub genre: String,
    pub style_prompt: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTest {
    pub ok: bool,
    pub message: String,
    pub models: Vec<String>,
}
