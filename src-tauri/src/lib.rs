use chrono::Utc;
use mw_memory::engine::MemoryEngine;
use regex::Regex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
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

#[derive(Debug, Deserialize)]
struct RememberCommandRequest {
    command_line: String,
    cwd: Option<String>,
    exit_code: Option<i64>,
    stdout: Option<String>,
    stderr: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommandRun {
    id: i64,
    command: String,
    argv_json: String,
    cwd: Option<String>,
    exit_code: Option<i64>,
    stdout: String,
    stderr: String,
    notes: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct CommandArgument {
    id: i64,
    command_run_id: i64,
    position: i64,
    value: String,
}

#[derive(Debug, Serialize)]
struct TerminalMemory {
    runs: Vec<CommandRun>,
    arguments: Vec<CommandArgument>,
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
            remember_command_run,
            list_terminal_memory,
            recall_memories,
            explain_memory,
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

        CREATE TABLE IF NOT EXISTS command_runs (
            id INTEGER PRIMARY KEY,
            command TEXT NOT NULL,
            argv_json TEXT NOT NULL,
            cwd TEXT,
            exit_code INTEGER,
            stdout TEXT NOT NULL DEFAULT '',
            stderr TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS command_arguments (
            id INTEGER PRIMARY KEY,
            command_run_id INTEGER NOT NULL,
            position INTEGER NOT NULL,
            value TEXT NOT NULL,
            FOREIGN KEY(command_run_id) REFERENCES command_runs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS agent_turns (
            id INTEGER PRIMARY KEY,
            session_id TEXT NOT NULL,
            ts TEXT NOT NULL,
            direction TEXT NOT NULL,
            verdict TEXT,
            text TEXT NOT NULL,
            cwd TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_agent_turns_session ON agent_turns(session_id);
        CREATE INDEX IF NOT EXISTS idx_command_runs_command ON command_runs(command);
        CREATE INDEX IF NOT EXISTS idx_command_runs_exit_code ON command_runs(exit_code);
        CREATE INDEX IF NOT EXISTS idx_command_arguments_value ON command_arguments(value);
        CREATE INDEX IF NOT EXISTS idx_documents_title ON documents(title);
        CREATE INDEX IF NOT EXISTS idx_concepts_name ON concepts(name);
        CREATE INDEX IF NOT EXISTS idx_links_from ON links(from_id);
        CREATE INDEX IF NOT EXISTS idx_links_to ON links(to_id);
        ",
    )?;
    Ok(conn)
}

fn database_path() -> anyhow::Result<PathBuf> {
    if let Some(path) = env::var_os("MEMORYWHALE_DATA_DIR") {
        return Ok(PathBuf::from(path).join("memorywhale.sqlite3"));
    }

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
    save_document(
        &state,
        ImportRequest {
            title: Some(title),
            source_type,
            content,
        },
    )
}

#[tauri::command]
fn import_text(
    state: tauri::State<AppState>,
    request: ImportRequest,
) -> Result<Document, AppError> {
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

    Ok(SearchResult {
        documents,
        concepts,
    })
}

#[tauri::command]
fn remember_command_run(
    state: tauri::State<AppState>,
    request: RememberCommandRequest,
) -> Result<CommandRun, AppError> {
    save_command_run(&state, request)
}

#[tauri::command]
fn list_terminal_memory(
    state: tauri::State<AppState>,
    query: Option<String>,
) -> Result<TerminalMemory, AppError> {
    let conn = state
        .db
        .lock()
        .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
    load_terminal_memory(&conn, query.as_deref())
}

// ── Explainable retrieval (mw-memory) ──────────────────────────────────────

#[derive(Debug, Serialize)]
struct SignalView {
    name: String,
    weight: f32,
    score: f32,
    applicable: bool,
    contribution: f32,
    detail: String,
}

/// A retrieved memory plus *why* it was retrieved — data behind the Recall panel.
#[derive(Debug, Serialize)]
struct RecallHit {
    id: i64,
    text: String,
    score: u32, // percent
    reasons: Vec<String>,
    signals: Vec<SignalView>,
    created_at: String,
    last_used: String,
    mentions: u32,
    importance: f32,
    tags: Vec<String>,
}

fn to_hit(sm: &mw_memory::ScoredMemory) -> RecallHit {
    RecallHit {
        id: sm.memory.id,
        text: sm.memory.text.clone(),
        score: sm.percent(),
        reasons: sm.reasons(),
        signals: sm
            .signals
            .iter()
            .map(|s| SignalView {
                name: s.name.clone(),
                weight: s.weight,
                score: s.score,
                applicable: s.applicable,
                contribution: s.contribution(),
                detail: s.detail.clone(),
            })
            .collect(),
        created_at: sm.memory.created_at.to_rfc3339(),
        last_used: sm.memory.last_used.to_rfc3339(),
        mentions: sm.memory.mentions,
        importance: sm.memory.importance,
        tags: sm.memory.tags.clone(),
    }
}

/// Load everything MemoryWhale remembers — documents (notes), command runs, and
/// agent conversation turns — as scorable memories. Ids are namespaced per source
/// (documents as-is, commands +1e9, conversation +2e9) so `explain(id)` is stable.
fn load_memories(conn: &Connection) -> Result<Vec<mw_memory::Memory>, AppError> {
    let parse = |ts: &str| {
        chrono::DateTime::parse_from_rfc3339(ts)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    };
    let mut mems: Vec<mw_memory::Memory> = Vec::new();

    // Documents / notes.
    let docs = conn
        .prepare("SELECT id, title, content, source_type, created_at FROM documents")?
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (id, title, content, source_type, created) in docs {
        let when = parse(&created);
        mems.push(mw_memory::Memory {
            id,
            text: format!("{title}. {content}"),
            created_at: when,
            last_used: when,
            mentions: 1,
            importance: 0.5,
            tags: vec!["document".into(), source_type],
            embedding: None,
        });
    }

    // Command runs — reinforcement = how often the same command recurs.
    let runs = conn
        .prepare("SELECT id, command, argv_json, notes, stderr, exit_code, created_at FROM command_runs")?
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, String>(6)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut cmd_counts: HashMap<String, u32> = HashMap::new();
    for (_, cmd, ..) in &runs {
        *cmd_counts.entry(cmd.to_lowercase()).or_insert(0) += 1;
    }
    for (id, command, argv_json, notes, stderr, exit_code, created) in runs {
        let when = parse(&created);
        let importance = if exit_code.unwrap_or(0) != 0 { 0.65 } else { 0.4 };
        mems.push(mw_memory::Memory {
            id: 1_000_000_000 + id,
            text: format!("{command} {argv_json} {notes} {stderr}"),
            created_at: when,
            last_used: when,
            mentions: *cmd_counts.get(&command.to_lowercase()).unwrap_or(&1),
            importance,
            tags: vec![
                "command".into(),
                if exit_code == Some(0) { "ok".into() } else { "error".into() },
            ],
            embedding: None,
        });
    }

    // Agent conversation turns (written by Delphin / hooks, if present).
    let turns = conn
        .prepare("SELECT id, ts, direction, text FROM agent_turns")?
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut turn_counts: HashMap<String, u32> = HashMap::new();
    for (_, _, _, t) in &turns {
        *turn_counts.entry(t.trim().to_lowercase()).or_insert(0) += 1;
    }
    for (id, ts, direction, text) in turns {
        if text.trim().is_empty() {
            continue;
        }
        let when = parse(&ts);
        let importance = match direction.as_str() {
            "user" => 0.6_f32,
            "agent" => 0.45,
            _ => 0.3,
        };
        let mentions = *turn_counts.get(&text.trim().to_lowercase()).unwrap_or(&1);
        mems.push(mw_memory::Memory {
            id: 2_000_000_000 + id,
            text,
            created_at: when,
            last_used: when,
            mentions,
            importance,
            tags: vec!["conversation".into(), direction],
            embedding: None,
        });
    }

    Ok(mems)
}

#[tauri::command]
fn recall_memories(
    state: tauri::State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<RecallHit>, AppError> {
    let mems = {
        let conn = state
            .db
            .lock()
            .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
        load_memories(&conn)?
    };
    let engine = mw_memory::engine::BuiltinEngine::new(mems);
    let q = mw_memory::Query::new(query, Utc::now());
    Ok(engine
        .retrieve(&q, limit.unwrap_or(8))
        .iter()
        .map(to_hit)
        .collect())
}

#[tauri::command]
fn explain_memory(
    state: tauri::State<AppState>,
    id: i64,
    query: String,
) -> Result<Option<RecallHit>, AppError> {
    let mems = {
        let conn = state
            .db
            .lock()
            .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
        load_memories(&conn)?
    };
    let engine = mw_memory::engine::BuiltinEngine::new(mems);
    let q = mw_memory::Query::new(query, Utc::now());
    Ok(engine.explain(id, &q).as_ref().map(to_hit))
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
        DELETE FROM command_arguments;
        DELETE FROM command_runs;
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

    let command_samples = [
        RememberCommandRequest {
            command_line: "npm run build".to_string(),
            cwd: Some("MemoryWhale".to_string()),
            exit_code: Some(0),
            stdout: Some("tsc && vite build completed successfully".to_string()),
            stderr: None,
            notes: Some("Baseline frontend build for the Rust/Tauri app.".to_string()),
        },
        RememberCommandRequest {
            command_line: "cargo check --manifest-path MemoryWhale/src-tauri/Cargo.toml"
                .to_string(),
            cwd: Some("MemoryWhale".to_string()),
            exit_code: Some(127),
            stdout: None,
            stderr: Some("zsh:1: command not found: cargo".to_string()),
            notes: Some(
                "Rust verification was blocked because cargo was unavailable in the terminal."
                    .to_string(),
            ),
        },
    ];

    for sample in command_samples {
        save_command_run(&state, sample)?;
    }

    let conn = state
        .db
        .lock()
        .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
    load_graph(&conn)
}

fn save_command_run(
    state: &tauri::State<AppState>,
    request: RememberCommandRequest,
) -> Result<CommandRun, AppError> {
    let argv = split_command_line(&request.command_line);
    let command = argv
        .first()
        .cloned()
        .unwrap_or_else(|| request.command_line.trim().to_string());
    if command.is_empty() {
        return Err(AppError::Message(
            "command line cannot be empty".to_string(),
        ));
    }

    let stdout = request.stdout.unwrap_or_default();
    let stderr = request.stderr.unwrap_or_default();
    let notes = request.notes.unwrap_or_default();
    let created_at = Utc::now().to_rfc3339();
    let argv_json = serde_json::to_string(&argv)
        .map_err(|err| AppError::Message(format!("failed to encode argv: {err}")))?;

    let mut conn = state
        .db
        .lock()
        .map_err(|_| AppError::Message("database lock poisoned".to_string()))?;
    let tx = conn.transaction()?;
    tx.execute(
        "
        INSERT INTO command_runs (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
            command,
            argv_json,
            request.cwd,
            request.exit_code,
            stdout,
            stderr,
            notes,
            created_at
        ],
    )?;
    let run_id = tx.last_insert_rowid();

    for (position, value) in argv.iter().enumerate() {
        tx.execute(
            "
            INSERT INTO command_arguments (command_run_id, position, value)
            VALUES (?1, ?2, ?3)
            ",
            params![run_id, position as i64, value],
        )?;
    }

    let combined = format!(
        "{}\n{}\n{}\n{}",
        request.command_line, stdout, stderr, notes
    );
    let command_node = format!("command:{run_id}");
    for concept in extract_keywords(&combined, 10) {
        tx.execute(
            "INSERT OR IGNORE INTO concepts (name, description) VALUES (?1, ?2)",
            params![
                concept,
                format!("Terminal memory concept extracted from command runs: {concept}")
            ],
        )?;
        let concept_id: i64 = tx.query_row(
            "SELECT id FROM concepts WHERE name = ?1",
            params![concept],
            |row| row.get(0),
        )?;
        upsert_link(
            &tx,
            &command_node,
            &format!("concept:{concept_id}"),
            "terminal_mentions",
        )?;
    }

    tx.commit()?;

    conn.query_row(
        "
        SELECT id, command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at
        FROM command_runs
        WHERE id = ?1
        ",
        params![run_id],
        row_to_command_run,
    )
    .map_err(AppError::from)
}

fn save_document(
    state: &tauri::State<AppState>,
    request: ImportRequest,
) -> Result<Document, AppError> {
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
        params![
            title,
            request.source_type,
            request.content,
            summary,
            created_at
        ],
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
            params![
                concept,
                format!("Recurring idea extracted from local sources: {concept}")
            ],
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

fn upsert_link(
    conn: &rusqlite::Transaction<'_>,
    from_id: &str,
    to_id: &str,
    relation: &str,
) -> rusqlite::Result<()> {
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
    let command_runs = load_terminal_memory(conn, None)?.runs;
    for run in &command_runs {
        let id = format!("command:{}", run.id);
        let status = match run.exit_code {
            Some(0) => "ok",
            Some(_) => "error",
            None => "unknown",
        };
        nodes.push(GraphNode {
            weight: *node_weights.get(&id).unwrap_or(&2),
            id,
            label: format!("{} ({status})", run.command),
            node_type: "command".to_string(),
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

fn load_terminal_memory(
    conn: &Connection,
    query: Option<&str>,
) -> Result<TerminalMemory, AppError> {
    let trimmed = query.unwrap_or("").trim();
    let runs = if trimmed.is_empty() {
        conn.prepare(
            "
            SELECT id, command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at
            FROM command_runs
            ORDER BY created_at DESC
            LIMIT 80
            ",
        )?
        .query_map([], row_to_command_run)?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        let pattern = format!("%{trimmed}%");
        conn.prepare(
            "
            SELECT DISTINCT cr.id, cr.command, cr.argv_json, cr.cwd, cr.exit_code,
                cr.stdout, cr.stderr, cr.notes, cr.created_at
            FROM command_runs cr
            LEFT JOIN command_arguments ca ON ca.command_run_id = cr.id
            WHERE cr.command LIKE ?1
                OR cr.argv_json LIKE ?1
                OR cr.cwd LIKE ?1
                OR cr.stdout LIKE ?1
                OR cr.stderr LIKE ?1
                OR cr.notes LIKE ?1
                OR ca.value LIKE ?1
            ORDER BY cr.created_at DESC
            LIMIT 80
            ",
        )?
        .query_map(params![pattern], row_to_command_run)?
        .collect::<Result<Vec<_>, _>>()?
    };

    let run_ids: HashSet<i64> = runs.iter().map(|run| run.id).collect();
    let mut all_args = conn
        .prepare(
            "
            SELECT id, command_run_id, position, value
            FROM command_arguments
            ORDER BY command_run_id DESC, position ASC
            ",
        )?
        .query_map([], row_to_command_argument)?
        .collect::<Result<Vec<_>, _>>()?;
    all_args.retain(|argument| run_ids.contains(&argument.command_run_id));

    Ok(TerminalMemory {
        runs,
        arguments: all_args,
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

fn row_to_command_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<CommandRun> {
    Ok(CommandRun {
        id: row.get(0)?,
        command: row.get(1)?,
        argv_json: row.get(2)?,
        cwd: row.get(3)?,
        exit_code: row.get(4)?,
        stdout: row.get(5)?,
        stderr: row.get(6)?,
        notes: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn row_to_command_argument(row: &rusqlite::Row<'_>) -> rusqlite::Result<CommandArgument> {
    Ok(CommandArgument {
        id: row.get(0)?,
        command_run_id: row.get(1)?,
        position: row.get(2)?,
        value: row.get(3)?,
    })
}

fn source_type_for_path(path: &Path) -> String {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "md" | "markdown" => "markdown".to_string(),
        "txt" => "text".to_string(),
        _ => "text".to_string(),
    }
}

fn split_command_line(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }
        if ch == '\'' || ch == '"' {
            quote = Some(ch);
            continue;
        }
        if ch.is_whitespace() {
            if !current.is_empty() {
                args.push(std::mem::take(&mut current));
            }
            while chars.peek().is_some_and(|next| next.is_whitespace()) {
                chars.next();
            }
            continue;
        }
        current.push(ch);
    }

    if !current.is_empty() {
        args.push(current);
    }
    args
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
        .map(|line| {
            line.trim_start_matches('>')
                .trim_matches('"')
                .trim()
                .to_string()
        })
        .filter(|line| line.len() > 24)
        .take(8)
        .collect()
}

fn extract_keywords(content: &str, limit: usize) -> Vec<String> {
    let stop_words: HashSet<&str> = [
        "about",
        "after",
        "again",
        "also",
        "and",
        "because",
        "been",
        "being",
        "between",
        "could",
        "each",
        "every",
        "from",
        "have",
        "into",
        "like",
        "more",
        "most",
        "notes",
        "over",
        "should",
        "that",
        "their",
        "there",
        "these",
        "they",
        "this",
        "through",
        "transcript",
        "using",
        "were",
        "when",
        "where",
        "which",
        "while",
        "with",
        "would",
        "your",
        "the",
        "for",
        "are",
        "can",
        "will",
        "all",
        "app",
        "local",
        "first",
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
            let technical_bonus = if word.contains('-') || word.contains('_') {
                2
            } else {
                0
            };
            let length_bonus = (word.len() as i64 / 7).min(2);
            (word, count * 3 + technical_bonus + length_bonus)
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored
        .into_iter()
        .take(limit)
        .map(|(word, _)| word)
        .collect()
}
