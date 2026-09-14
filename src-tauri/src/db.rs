use crate::models::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn nid() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn open(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(map_err)?;
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            genre TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            style_prompt TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS characters (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            name TEXT NOT NULL,
            aliases TEXT NOT NULL DEFAULT '',
            role TEXT NOT NULL DEFAULT '',
            personality TEXT NOT NULL DEFAULT '',
            appearance TEXT NOT NULL DEFAULT '',
            background TEXT NOT NULL DEFAULT '',
            goals TEXT NOT NULL DEFAULT '',
            relationships TEXT NOT NULL DEFAULT '',
            current_state TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS entries (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL,
            category TEXT NOT NULL DEFAULT '设定',
            content TEXT NOT NULL DEFAULT '',
            tags TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS outlines (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            parent_id TEXT,
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            order_index INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS chapters (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            outline_id TEXT,
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            summary TEXT NOT NULL DEFAULT '',
            order_index INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT 'plot',
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )
    .map_err(map_err)?;
    Ok(conn)
}

fn map_err(err: impl ToString) -> String {
    err.to_string()
}

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>, String> {
    let mut stmt = conn
        .prepare("SELECT id, title, genre, description, style_prompt, created_at, updated_at FROM projects ORDER BY updated_at DESC")
        .map_err(map_err)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                title: row.get(1)?,
                genre: row.get(2)?,
                description: row.get(3)?,
                style_prompt: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(map_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn get_project(conn: &Connection, id: &str) -> Result<Option<Project>, String> {
    conn.query_row(
        "SELECT id, title, genre, description, style_prompt, created_at, updated_at FROM projects WHERE id = ?1",
        [id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                title: row.get(1)?,
                genre: row.get(2)?,
                description: row.get(3)?,
                style_prompt: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .optional()
    .map_err(map_err)
}

pub fn upsert_project(conn: &Connection, mut project: Project) -> Result<Project, String> {
    let ts = now();
    if project.id.is_empty() {
        project.id = nid();
        project.created_at = ts.clone();
    }
    if project.created_at.is_empty() {
        project.created_at = ts.clone();
    }
    project.updated_at = ts;
    conn.execute(
        r#"
        INSERT INTO projects (id, title, genre, description, style_prompt, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            genre = excluded.genre,
            description = excluded.description,
            style_prompt = excluded.style_prompt,
            updated_at = excluded.updated_at
        "#,
        params![
            project.id,
            project.title,
            project.genre,
            project.description,
            project.style_prompt,
            project.created_at,
            project.updated_at
        ],
    )
    .map_err(map_err)?;
    Ok(project)
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM projects WHERE id = ?1", [id])
        .map_err(map_err)?;
    Ok(())
}

pub fn list_characters(conn: &Connection, project_id: &str) -> Result<Vec<Character>, String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT id, project_id, name, aliases, role, personality, appearance, background,
                      goals, relationships, current_state, notes, created_at, updated_at
               FROM characters WHERE project_id = ?1 ORDER BY updated_at DESC"#,
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map([project_id], |row| {
            Ok(Character {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                aliases: row.get(3)?,
                role: row.get(4)?,
                personality: row.get(5)?,
                appearance: row.get(6)?,
                background: row.get(7)?,
                goals: row.get(8)?,
                relationships: row.get(9)?,
                current_state: row.get(10)?,
                notes: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })
        .map_err(map_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn upsert_character(conn: &Connection, mut item: Character) -> Result<Character, String> {
    let ts = now();
    if item.id.is_empty() {
        item.id = nid();
        item.created_at = ts.clone();
    }
    if item.created_at.is_empty() {
        item.created_at = ts.clone();
    }
    item.updated_at = ts;
    conn.execute(
        r#"
        INSERT INTO characters (id, project_id, name, aliases, role, personality, appearance, background,
            goals, relationships, current_state, notes, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        ON CONFLICT(id) DO UPDATE SET
            name=excluded.name, aliases=excluded.aliases, role=excluded.role,
            personality=excluded.personality, appearance=excluded.appearance,
            background=excluded.background, goals=excluded.goals,
            relationships=excluded.relationships, current_state=excluded.current_state,
            notes=excluded.notes, updated_at=excluded.updated_at
        "#,
        params![
            item.id,
            item.project_id,
            item.name,
            item.aliases,
            item.role,
            item.personality,
            item.appearance,
            item.background,
            item.goals,
            item.relationships,
            item.current_state,
            item.notes,
            item.created_at,
            item.updated_at
        ],
    )
    .map_err(map_err)?;
    touch_project(conn, &item.project_id)?;
    Ok(item)
}

pub fn delete_character(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM characters WHERE id = ?1", [id])
        .map_err(map_err)?;
    Ok(())
}

pub fn list_entries(conn: &Connection, project_id: &str) -> Result<Vec<Entry>, String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT id, project_id, title, category, content, tags, created_at, updated_at
               FROM entries WHERE project_id = ?1 ORDER BY updated_at DESC"#,
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map([project_id], |row| {
            Ok(Entry {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                category: row.get(3)?,
                content: row.get(4)?,
                tags: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(map_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn upsert_entry(conn: &Connection, mut item: Entry) -> Result<Entry, String> {
    let ts = now();
    if item.id.is_empty() {
        item.id = nid();
        item.created_at = ts.clone();
    }
    if item.created_at.is_empty() {
        item.created_at = ts.clone();
    }
    item.updated_at = ts;
    conn.execute(
        r#"
        INSERT INTO entries (id, project_id, title, category, content, tags, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            title=excluded.title, category=excluded.category, content=excluded.content,
            tags=excluded.tags, updated_at=excluded.updated_at
        "#,
        params![
            item.id,
            item.project_id,
            item.title,
            item.category,
            item.content,
            item.tags,
            item.created_at,
            item.updated_at
        ],
    )
    .map_err(map_err)?;
    touch_project(conn, &item.project_id)?;
    Ok(item)
}

pub fn delete_entry(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM entries WHERE id = ?1", [id])
        .map_err(map_err)?;
    Ok(())
}

pub fn list_outlines(conn: &Connection, project_id: &str) -> Result<Vec<Outline>, String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT id, project_id, parent_id, title, content, order_index, status, created_at, updated_at
               FROM outlines WHERE project_id = ?1 ORDER BY order_index ASC, created_at ASC"#,
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map([project_id], |row| {
            Ok(Outline {
                id: row.get(0)?,
                project_id: row.get(1)?,
                parent_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                order_index: row.get(5)?,
                status: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(map_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn upsert_outline(conn: &Connection, mut item: Outline) -> Result<Outline, String> {
    let ts = now();
    if item.id.is_empty() {
        item.id = nid();
        item.created_at = ts.clone();
    }
    if item.created_at.is_empty() {
        item.created_at = ts.clone();
    }
    item.updated_at = ts;
    if item.parent_id.as_deref().is_some_and(|v| v.is_empty()) {
        item.parent_id = None;
    }
    conn.execute(
        r#"
        INSERT INTO outlines (id, project_id, parent_id, title, content, order_index, status, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(id) DO UPDATE SET
            parent_id=excluded.parent_id, title=excluded.title, content=excluded.content,
            order_index=excluded.order_index, status=excluded.status, updated_at=excluded.updated_at
        "#,
        params![
            item.id,
            item.project_id,
            item.parent_id,
            item.title,
            item.content,
            item.order_index,
            item.status,
            item.created_at,
            item.updated_at
        ],
    )
    .map_err(map_err)?;
    touch_project(conn, &item.project_id)?;
    Ok(item)
}

pub fn delete_outline(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM outlines WHERE id = ?1", [id])
        .map_err(map_err)?;
    Ok(())
}

pub fn list_chapters(conn: &Connection, project_id: &str) -> Result<Vec<Chapter>, String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT id, project_id, outline_id, title, content, summary, order_index, status, created_at, updated_at
               FROM chapters WHERE project_id = ?1 ORDER BY order_index ASC, created_at ASC"#,
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map([project_id], |row| {
            Ok(Chapter {
                id: row.get(0)?,
                project_id: row.get(1)?,
                outline_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                order_index: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(map_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn get_chapter(conn: &Connection, id: &str) -> Result<Option<Chapter>, String> {
    conn.query_row(
        r#"SELECT id, project_id, outline_id, title, content, summary, order_index, status, created_at, updated_at
           FROM chapters WHERE id = ?1"#,
        [id],
        |row| {
            Ok(Chapter {
                id: row.get(0)?,
                project_id: row.get(1)?,
                outline_id: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                order_index: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
    .optional()
    .map_err(map_err)
}

pub fn upsert_chapter(conn: &Connection, mut item: Chapter) -> Result<Chapter, String> {
    let ts = now();
    if item.id.is_empty() {
        item.id = nid();
        item.created_at = ts.clone();
    }
    if item.created_at.is_empty() {
        item.created_at = ts.clone();
    }
    item.updated_at = ts;
    if item.outline_id.as_deref().is_some_and(|v| v.is_empty()) {
        item.outline_id = None;
    }
    conn.execute(
        r#"
        INSERT INTO chapters (id, project_id, outline_id, title, content, summary, order_index, status, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(id) DO UPDATE SET
            outline_id=excluded.outline_id, title=excluded.title, content=excluded.content,
            summary=excluded.summary, order_index=excluded.order_index, status=excluded.status,
            updated_at=excluded.updated_at
        "#,
        params![
            item.id,
            item.project_id,
            item.outline_id,
            item.title,
            item.content,
            item.summary,
            item.order_index,
            item.status,
            item.created_at,
            item.updated_at
        ],
    )
    .map_err(map_err)?;
    touch_project(conn, &item.project_id)?;
    Ok(item)
}

pub fn delete_chapter(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM chapters WHERE id = ?1", [id])
        .map_err(map_err)?;
    Ok(())
}

pub fn list_memories(conn: &Connection, project_id: &str) -> Result<Vec<Memory>, String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT id, project_id, kind, title, content, source, created_at, updated_at
               FROM memories WHERE project_id = ?1 ORDER BY updated_at DESC"#,
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map([project_id], |row| {
            Ok(Memory {
                id: row.get(0)?,
                project_id: row.get(1)?,
                kind: row.get(2)?,
                title: row.get(3)?,
                content: row.get(4)?,
                source: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(map_err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn upsert_memory(conn: &Connection, mut item: Memory) -> Result<Memory, String> {
    let ts = now();
    if item.id.is_empty() {
        item.id = nid();
        item.created_at = ts.clone();
    }
    if item.created_at.is_empty() {
        item.created_at = ts.clone();
    }
    item.updated_at = ts;
    conn.execute(
        r#"
        INSERT INTO memories (id, project_id, kind, title, content, source, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            kind=excluded.kind, title=excluded.title, content=excluded.content,
            source=excluded.source, updated_at=excluded.updated_at
        "#,
        params![
            item.id,
            item.project_id,
            item.kind,
            item.title,
            item.content,
            item.source,
            item.created_at,
            item.updated_at
        ],
    )
    .map_err(map_err)?;
    touch_project(conn, &item.project_id)?;
    Ok(item)
}

pub fn delete_memory(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM memories WHERE id = ?1", [id])
        .map_err(map_err)?;
    Ok(())
}

pub fn load_bundle(conn: &Connection, project_id: &str) -> Result<ProjectBundle, String> {
    let project = get_project(conn, project_id)?.ok_or_else(|| "书稿不存在".to_string())?;
    Ok(ProjectBundle {
        project,
        characters: list_characters(conn, project_id)?,
        entries: list_entries(conn, project_id)?,
        outlines: list_outlines(conn, project_id)?,
        chapters: list_chapters(conn, project_id)?,
        memories: list_memories(conn, project_id)?,
    })
}

pub fn get_settings(conn: &Connection) -> Result<AppSettings, String> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'app'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(map_err)?;
    match raw {
        Some(json) => serde_json::from_str(&json).map_err(map_err),
        None => Ok(AppSettings::default()),
    }
}

pub fn save_settings(conn: &Connection, settings: &AppSettings) -> Result<(), String> {
    let json = serde_json::to_string(settings).map_err(map_err)?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('app', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [json],
    )
    .map_err(map_err)?;
    Ok(())
}

fn touch_project(conn: &Connection, project_id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE projects SET updated_at = ?1 WHERE id = ?2",
        params![now(), project_id],
    )
    .map_err(map_err)?;
    Ok(())
}
