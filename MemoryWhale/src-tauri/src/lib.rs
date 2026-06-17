use chrono::Utc;
use regex::Regex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

struct AppState {
    db: Mutex<Connection>,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Deserialize)]
struct ImportRequest {
    title: Option<String>,
    source_type: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct Document {
    id: i64,
    title: String,
    source_type: String,
    content: String,
    summary: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct Concept {
    id: i64,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct Quote {
    id: i64,
    document_id: i64,
    text: String,
}

#[derive(Debug, Serialize)]
struct GraphNode {
    id: String,
    label: String,
    node_type: String,
    weight: i64,
}

#[derive(Debug, Serialize)]
struct GraphLink {
    source: String,
    target: String,
    relation: String,
    weight: i64,
}

#[derive(Debug, Serialize)]
struct GraphPayload {
    documents: Vec<Document>,
    concepts: Vec<Concept>,
    quotes: Vec<Quote>,
    nodes: Vec<GraphNode>,
    links: Vec<GraphLink>,
}

#[derive(Debug, Serialize)]
struct SearchResult {
    documents: Vec<Document>,
    concepts: Vec<Concept>,
}

pub fn run() {
    let db = init_connection().expect("failed to initialize MemoryWhale database");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            import_text,
            import_file,
            get_graph,
            search_memory,
            reset_demo_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running MemoryWhale");
}

fn init_connection() -> anyhow::Result<Connection> {
    let db_path = database_path()?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            source_type TEXT NOT NULL,
            content TEXT NOT NULL,
            summary TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY,
            document_id INTEGER NOT NULL,
            body TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(document_id) REFERENCES documents(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS concepts (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT
        );

        CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY,
            from_id TEXT NOT NULL,
            to_id TEXT NOT NULL,
            relation TEXT NOT NULL,
            weight INTEGER NOT NULL DEFAULT 1,
            UNIQUE(from_id, to_id, relation)
        );

        CREATE TABLE IF NOT EXISTS quotes (
            id INTEGER PRIMARY KEY,
            document_id INTEGER NOT NULL,
            text TEXT NOT NULL,
            FOREIGN KEY(document_id) REFERENCES documents(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );

        CREATE INDEX IF NOT EXISTS idx_documents_title ON documents(title);
        CREATE INDEX IF NOT EXISTS idx_concepts_name ON concepts(name);
        CREATE INDEX IF NOT EXISTS idx_links_from ON links(from_id);
        CREATE INDEX IF NOT EXISTS idx_links_to ON links(to_id);
        ",
    )?;
    Ok(conn)
}

fn database_path() -> anyhow::Result<PathBuf> {
    let base = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| anyhow::anyhow!("could not resolve a local data directory"))?;
    Ok(base.join("MemoryWhale").join("memorywhale.sqlite3"))
}

#[tauri::command]
fn import_file(state: tauri::State<AppState>, path: String) -> Result<Document, AppError> {
    let path_buf = PathBuf::from(path);
    let content = fs::read_to_string(&path_buf)?;
    let title = path_buf
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Imported document")
        .to_string();
    let source_type = source_type_for_path(&path_buf);
    save_document(&state, ImportRequest {
        title: Some(title),
        source_type,
        content,
    })
}

#[tauri::command]
fn import_text(state: tauri::State<AppState>, request: ImportRequest) -> Result<Document, AppError> {
    save_document(&state, request)
}

#[tauri::command]
fn get_graph(state: tauri::State<AppState>) -> Result<GraphPayload, AppError> {
    let conn = state
        .db
        .lock()
        .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
    load_graph(&conn)
}

#[tauri::command]
fn search_memory(state: tauri::State<AppState>, query: String) -> Result<SearchResult, AppError> {
    let conn = state
        .db
        .lock()
        .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
    let pattern = format!("%{}%", query.trim());

    let documents = conn
        .prepare(
            "
            SELECT id, title, source_type, content, summary, created_at
            FROM documents
            WHERE title LIKE ?1 OR content LIKE ?1 OR summary LIKE ?1
            ORDER BY created_at DESC
            LIMIT 30
            ",
        )?
        .query_map(params![pattern], row_to_document)?
        .collect::<Result<Vec<_>, _>>()?;

    let concept_pattern = format!("%{}%", query.trim().to_lowercase());
    let concepts = conn
        .prepare(
            "
            SELECT id, name, description
            FROM concepts
            WHERE name LIKE ?1 OR description LIKE ?1
            ORDER BY name
            LIMIT 30
            ",
        )?
        .query_map(params![concept_pattern], row_to_concept)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SearchResult { documents, concepts })
}

#[tauri::command]
fn reset_demo_data(state: tauri::State<AppState>) -> Result<GraphPayload, AppError> {
    let conn = state
        .db
        .lock()
        .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
    conn.execute_batch(
        "
        DELETE FROM quotes;
        DELETE FROM links;
        DELETE FROM concepts;
        DELETE FROM notes;
        DELETE FROM documents;
        ",
    )?;
    drop(conn);

    let samples = [
        ImportRequest {
            title: Some("Rust Desktop Systems".to_string()),
            source_type: "markdown".to_string(),
            content: "Rust, Tauri, and SQLite make local-first desktop software feel fast. Tauri keeps the shell small while Rust handles parsing, persistence, and safe commands.".to_string(),
        },
        ImportRequest {
            title: Some("Knowledge Galaxy Notes".to_string()),
            source_type: "note".to_string(),
            content: "A knowledge graph can connect documents, concepts, quotes, and tags. Big nodes should represent recurring topics. Search should reveal everything related to robotics, Rust, or graph visualization.".to_string(),
        },
        ImportRequest {
            title: Some("Transcript: Memory Tools".to_string()),
            source_type: "youtube_transcript".to_string(),
            content: "NotebookLM and Obsidian show how useful connected notes can be. MemoryWhale should import transcripts, extract concepts, and help people explore related ideas without sending private data to the cloud.".to_string(),
        },
    ];

    for sample in samples {
        save_document(&state, sample)?;
    }

    let conn = state
        .db
        .lock()
        .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
    load_graph(&conn)
}

fn save_document(state: &tauri::State<AppState>, request: ImportRequest) -> Result<Document, AppError> {
    let title = request
        .title
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| infer_title(&request.content));
    let summary = summarize(&request.content);
    let created_at = Utc::now().to_rfc3339();
    let concepts = extract_keywords(&request.content, 12);
    let quotes = extract_quotes(&request.content);

    let mut conn = state
        .db
        .lock()
        .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
    let tx = conn.transaction()?;
    tx.execute(
        "
        INSERT INTO documents (title, source_type, content, summary, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ",
        params![title, request.source_type, request.content, summary, created_at],
    )?;
    let document_id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO notes (document_id, body, created_at) VALUES (?1, ?2, ?3)",
        params![document_id, summary, created_at],
    )?;

    for quote in quotes {
        tx.execute(
            "INSERT INTO quotes (document_id, text) VALUES (?1, ?2)",
            params![document_id, quote],
        )?;
    }

    let doc_node = format!("document:{document_id}");
    let mut concept_ids = Vec::new();
    for concept in concepts {
        tx.execute(
            "INSERT OR IGNORE INTO concepts (name, description) VALUES (?1, ?2)",
            params![concept, format!("Recurring idea extracted from local sources: {concept}")],
        )?;
        let concept_id: i64 = tx.query_row(
            "SELECT id FROM concepts WHERE name = ?1",
            params![concept],
            |row| row.get(0),
        )?;
        let concept_node = format!("concept:{concept_id}");
        concept_ids.push(concept_node.clone());
        upsert_link(&tx, &doc_node, &concept_node, "mentions")?;
    }

    for i in 0..concept_ids.len() {
        for other in concept_ids.iter().skip(i + 1) {
            upsert_link(&tx, &concept_ids[i], other, "co_occurs")?;
        }
    }

    tx.commit()?;

    let conn = state
        .db
        .lock()
        .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
    conn.query_row(
        "
        SELECT id, title, source_type, content, summary, created_at
        FROM documents
        WHERE id = ?1
        ",
        params![document_id],
        row_to_document,
    )
    .map_err(AppError::from)
}

fn upsert_link(conn: &Connection, from_id: &str, to_id: &str, relation: &str) -> rusqlite::Result<()> {
    conn.execute(
        "
        INSERT INTO links (from_id, to_id, relation, weight)
        VALUES (?1, ?2, ?3, 1)
        ON CONFLICT(from_id, to_id, relation)
        DO UPDATE SET weight = weight + 1
        ",
        params![from_id, to_id, relation],
    )?;
    Ok(())
}

fn load_graph(conn: &Connection) -> Result<GraphPayload, AppError> {
    let documents = conn
        .prepare(
            "
            SELECT id, title, source_type, content, summary, created_at
            FROM documents
            ORDER BY created_at DESC
            ",
        )?
        .query_map([], row_to_document)?
        .collect::<Result<Vec<_>, _>>()?;

    let concepts = conn
        .prepare("SELECT id, name, description FROM concepts ORDER BY name")?
        .query_map([], row_to_concept)?
        .collect::<Result<Vec<_>, _>>()?;

    let quotes = conn
        .prepare("SELECT id, document_id, text FROM quotes ORDER BY id DESC LIMIT 100")?
        .query_map([], |row| {
            Ok(Quote {
                id: row.get(0)?,
                document_id: row.get(1)?,
                text: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut node_weights: HashMap<String, i64> = HashMap::new();
    let links = conn
        .prepare("SELECT from_id, to_id, relation, weight FROM links")?
        .query_map([], |row| {
            let source: String = row.get(0)?;
            let target: String = row.get(1)?;
            let weight: i64 = row.get(3)?;
            *node_weights.entry(source.clone()).or_insert(1) += weight;
            *node_weights.entry(target.clone()).or_insert(1) += weight;
            Ok(GraphLink {
                source,
                target,
                relation: row.get(2)?,
                weight,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut nodes = Vec::new();
    for doc in &documents {
        let id = format!("document:{}", doc.id);
        nodes.push(GraphNode {
            weight: *node_weights.get(&id).unwrap_or(&2),
            id,
            label: doc.title.clone(),
            node_type: "document".to_string(),
        });
    }
    for concept in &concepts {
        let id = format!("concept:{}", concept.id);
        nodes.push(GraphNode {
            weight: *node_weights.get(&id).unwrap_or(&1),
            id,
            label: concept.name.clone(),
            node_type: "concept".to_string(),
        });
    }

    Ok(GraphPayload {
        documents,
        concepts,
        quotes,
        nodes,
        links,
    })
}

fn row_to_document(row: &rusqlite::Row<'_>) -> rusqlite::Result<Document> {
    Ok(Document {
        id: row.get(0)?,
        title: row.get(1)?,
        source_type: row.get(2)?,
        content: row.get(3)?,
        summary: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn row_to_concept(row: &rusqlite::Row<'_>) -> rusqlite::Result<Concept> {
    Ok(Concept {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
    })
}

fn source_type_for_path(path: &Path) -> String {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_lowercase().as_str() {
        "md" | "markdown" => "markdown".to_string(),
        "txt" => "text".to_string(),
        _ => "text".to_string(),
    }
}

fn infer_title(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim().trim_start_matches('#').trim())
        .find(|line| !line.is_empty())
        .map(|line| line.chars().take(80).collect())
        .unwrap_or_else(|| "Untitled memory".to_string())
}

fn summarize(content: &str) -> String {
    let sentence_re = Regex::new(r"(?m)([^.!?\n]{32,220}[.!?])").unwrap();
    sentence_re
        .captures_iter(content)
        .take(3)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(520)
        .collect()
}

fn extract_quotes(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('>') || line.starts_with('"'))
        .map(|line| line.trim_start_matches('>').trim_matches('"').trim().to_string())
        .filter(|line| line.len() > 24)
        .take(8)
        .collect()
}

fn extract_keywords(content: &str, limit: usize) -> Vec<String> {
    let stop_words: HashSet<&str> = [
        "about", "after", "again", "also", "and", "because", "been", "being", "between",
        "could", "each", "every", "from", "have", "into", "like", "more", "most", "notes",
        "over", "should", "that", "their", "there", "these", "they", "this", "through",
        "transcript", "using", "were", "when", "where", "which", "while", "with", "would",
        "your", "the", "for", "are", "can", "will", "all", "app", "local", "first",
    ]
    .into_iter()
    .collect();
    let word_re = Regex::new(r"[A-Za-z][A-Za-z0-9_\-]{2,}").unwrap();
    let mut counts: HashMap<String, i64> = HashMap::new();

    for mat in word_re.find_iter(content) {
        let word = mat.as_str().to_lowercase();
        if stop_words.contains(word.as_str()) {
            continue;
        }
        *counts.entry(word).or_insert(0) += 1;
    }

    let mut scored: Vec<(String, i64)> = counts
        .into_iter()
        .map(|(word, count)| {
            let technical_bonus = if word.contains('-') || word.contains('_') { 2 } else { 0 };
            let length_bonus = (word.len() as i64 / 7).min(2);
            (word, count * 3 + technical_bonus + length_bonus)
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored.into_iter().take(limit).map(|(word, _)| word).collect()
}
