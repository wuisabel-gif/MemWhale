//! `mw tui` — an interactive terminal browser for your MemoryWhale memory.
//!
//! Type to search (ranked live by the same engine as `mw search`), arrow keys
//! (or Ctrl-n/Ctrl-p) to move, Enter to reveal the replay/show command for the
//! selected item, Esc or Ctrl-c to quit. Runs entirely in the terminal — no
//! browser, no server.

use chrono::{DateTime, Utc};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;
use rusqlite::Connection;

use memorywhale_core::engine::{BuiltinEngine, MemoryEngine};
use memorywhale_core::sqlite::{decode_id, load_memories, Source};
use memorywhale_core::{Query, ScoredMemory};

/// How many results to hold in the list — plenty for browsing, cheap to rank.
const MAX_RESULTS: usize = 500;

struct App {
    engine: BuiltinEngine,
    /// Live DB handle — used by review mode to approve/reject pending notes.
    conn: Connection,
    now: DateTime<Utc>,
    query: String,
    results: Vec<ScoredMemory>,
    state: ListState,
    /// The last "reveal" action (the shell command for the selected memory).
    status: String,
    /// Whether the F1 help overlay is open.
    show_help: bool,
    /// Whether the agent-memory review pane is showing (Tab toggles it).
    review: bool,
    /// Agent-written memories awaiting approval (`approved = 0`).
    pending: Vec<crate::PendingNote>,
    review_state: ListState,
}

impl App {
    fn new(engine: BuiltinEngine, conn: Connection) -> Self {
        let mut app = App {
            engine,
            conn,
            now: Utc::now(),
            query: String::new(),
            results: Vec::new(),
            state: ListState::default(),
            status: String::new(),
            show_help: false,
            review: false,
            pending: Vec::new(),
            review_state: ListState::default(),
        };
        app.recompute();
        app.refresh_pending();
        app
    }

    /// Reload the pending-review queue and keep the selection in range.
    fn refresh_pending(&mut self) {
        self.pending = crate::pending_agent_notes(&self.conn).unwrap_or_default();
        let sel = self
            .review_state
            .selected()
            .unwrap_or(0)
            .min(self.pending.len().saturating_sub(1));
        self.review_state
            .select((!self.pending.is_empty()).then_some(sel));
    }

    fn toggle_review(&mut self) {
        self.review = !self.review;
        if self.review {
            self.refresh_pending();
        }
        self.status.clear();
    }

    fn move_review_sel(&mut self, delta: isize) {
        if self.pending.is_empty() {
            return;
        }
        let len = self.pending.len() as isize;
        let cur = self.review_state.selected().unwrap_or(0) as isize;
        self.review_state
            .select(Some((cur + delta).rem_euclid(len) as usize));
    }

    /// Approve (`a`) or reject (`d`) the selected pending note, then refresh.
    fn review_action(&mut self, approve: bool) {
        let Some(note) = self
            .review_state
            .selected()
            .and_then(|i| self.pending.get(i))
        else {
            return;
        };
        let id = note.id;
        let result = if approve {
            crate::approve_note(&self.conn, id)
        } else {
            crate::reject_note(&self.conn, id)
        };
        self.status = match result {
            Ok(()) if approve => format!("  approved #{id}"),
            Ok(()) => format!("  rejected #{id}"),
            Err(e) => format!("  {e}"),
        };
        self.refresh_pending();
    }

    /// Re-rank against the current query. Empty query ranks by recency/importance
    /// (a "recent memory" browse). A non-empty query both *reranks* by relevance
    /// and *narrows* to memories that actually contain a query term — otherwise
    /// the engine returns everything (just reordered), which reads wrong for a
    /// search box.
    fn recompute(&mut self) {
        let terms: Vec<&str> = self.query.split_whitespace().collect();
        let (filters, search_text) = match crate::parse_search_filters(&terms) {
            Ok(parsed) => parsed,
            Err(error) => {
                self.results.clear();
                self.state.select(None);
                self.status = format!("  {error}");
                return;
            }
        };
        // Apply metadata filters before ranking/truncating so an agent match
        // cannot be hidden behind MAX_RESULTS higher-ranked other agents.
        let filtered = crate::filter_memories(self.engine.memories.clone(), &filters);
        let q = Query::new(&search_text, self.now);
        let engine = BuiltinEngine::new(filtered);
        let mut hits = engine.retrieve(&q, MAX_RESULTS);
        let terms: Vec<String> = search_text
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();
        if !terms.is_empty() {
            hits.retain(|sm| {
                let text = sm.memory.text.to_lowercase();
                terms.iter().any(|t| text.contains(t))
            });
        }
        self.results = hits;
        self.state.select((!self.results.is_empty()).then_some(0));
        self.status.clear();
    }

    fn move_sel(&mut self, delta: isize) {
        if self.results.is_empty() {
            return;
        }
        let len = self.results.len() as isize;
        let cur = self.state.selected().unwrap_or(0) as isize;
        self.state
            .select(Some((cur + delta).rem_euclid(len) as usize));
    }

    fn selected(&self) -> Option<&ScoredMemory> {
        self.state.selected().and_then(|i| self.results.get(i))
    }

    /// Show the shell command that acts on the selected memory (its whole point:
    /// browse here, then run it in your shell).
    fn reveal_action(&mut self) {
        if let Some(sm) = self.selected() {
            let (source, id) = decode_id(sm.memory.id);
            self.status = match source {
                Source::Command => format!("  ⮑ replay it:  mw replay {id}"),
                Source::Session => format!("  ⮑ open it:  mw show {id}"),
                Source::Note => format!("  ⮑ remembered note #{id}"),
                other => format!("  ⮑ {} #{id}", other.tag()),
            };
        }
    }
}

/// First non-empty line of a memory, trimmed and capped for a list row.
fn snippet(text: &str, max: usize) -> String {
    text.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
        .chars()
        .take(max)
        .collect()
}

fn render(app: &mut App, f: &mut Frame) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        // search box · body · status line · key bar
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Search box (doubles as a mode banner in review mode).
    let search = if app.review {
        Paragraph::new(Line::from(vec![Span::styled(
            "REVIEW MODE — approve/reject agent-written memories",
            Style::default().fg(Color::Yellow),
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" MemoryWhale "),
        )
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled("search ", Style::default().fg(Color::Cyan)),
            Span::raw(&app.query),
            Span::styled("▏", Style::default().fg(Color::Cyan)), // cursor
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" MemoryWhale "),
        )
    };
    f.render_widget(search, rows[0]);

    if app.review {
        render_review(app, f, rows[1]);
        render_status_and_keys(app, f, &rows);
        if app.show_help {
            render_help(f);
        }
        return;
    }

    // Body: results list | detail preview.
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(rows[1]);

    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|sm| {
            let (source, _) = decode_id(sm.memory.id);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>3}% ", sm.percent()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(
                        "{:<8} {:<8} ",
                        source.tag(),
                        memorywhale_core::provenance::label(sm.memory.agent.as_deref())
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(snippet(&sm.memory.text, 48)),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} results ", app.results.len())),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, body[0], &mut app.state);

    // Detail of the selected memory.
    let detail = match app.selected() {
        None => {
            Paragraph::new("(no matches — try a different search, or capture some commands first)")
        }
        Some(sm) => {
            let (source, id) = decode_id(sm.memory.id);
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(
                        format!(
                            "[{} · {}] ",
                            source.tag(),
                            memorywhale_core::provenance::label(sm.memory.agent.as_deref())
                        ),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(format!("#{id}  "), Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}% match", sm.percent()),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(""),
            ];
            for line in sm.memory.text.lines() {
                lines.push(Line::raw(line.to_string()));
            }
            let reasons = sm.reasons();
            if !reasons.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::styled(
                    "why this ranked here:",
                    Style::default().fg(Color::DarkGray),
                ));
                for r in reasons {
                    lines.push(Line::styled(
                        format!("  • {r}"),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }
            Paragraph::new(lines).wrap(Wrap { trim: false })
        }
    };
    f.render_widget(
        detail.block(Block::default().borders(Borders::ALL).title(" detail ")),
        body[1],
    );

    render_status_and_keys(app, f, &rows);

    // F1 help overlay, drawn last so it sits on top.
    if app.show_help {
        render_help(f);
    }
}

/// The pending-review pane: a list of unapproved agent memories with their
/// provenance, or a clear empty-state when the queue is empty (which is also
/// what you see when review mode isn't enabled — agent notes get auto-approved).
fn render_review(app: &mut App, f: &mut Frame, area: ratatui::layout::Rect) {
    if app.pending.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::styled(
                "  no memories awaiting review",
                Style::default().fg(Color::DarkGray),
            ),
            Line::styled(
                "  (agent memories land here when review_agent_memories = true)",
                Style::default().fg(Color::DarkGray),
            ),
        ])
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" pending review "),
        );
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .pending
        .iter()
        .map(|note| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!("#{:<4} ", note.id),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(snippet(&note.label, 60)),
                ]),
                Line::styled(
                    format!("      {}", note.provenance()),
                    Style::default().fg(Color::Yellow),
                ),
            ])
        })
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " {} awaiting review — a approve · d reject ",
            app.pending.len()
        )))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, &mut app.review_state);
}

/// Shared bottom chrome: the status line plus the always-visible key bar.
fn render_status_and_keys(app: &App, f: &mut Frame, rows: &[ratatui::layout::Rect]) {
    let status = if !app.status.is_empty() {
        Line::styled(app.status.clone(), Style::default().fg(Color::Green))
    } else if app.review {
        Line::styled(
            "  a approve · d reject the selected memory · Tab back to search",
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Line::styled(
            "  press Enter on a result to get its shell command",
            Style::default().fg(Color::DarkGray),
        )
    };
    f.render_widget(Paragraph::new(status), rows[2]);
    f.render_widget(Paragraph::new(key_bar(app.review)), rows[3]);
}

/// The always-on key hints. Full list lives in the F1 overlay.
fn key_bar(review: bool) -> Line<'static> {
    let key = |k: &'static str| Span::styled(k, Style::default().fg(Color::Cyan));
    let sep = || Span::styled("  ·  ", Style::default().fg(Color::DarkGray));
    if review {
        return Line::from(vec![
            Span::raw(" "),
            key("a"),
            Span::raw(" approve"),
            sep(),
            key("d"),
            Span::raw(" reject"),
            sep(),
            key("↑↓"),
            Span::raw(" move"),
            sep(),
            key("Tab"),
            Span::raw(" back to search"),
            sep(),
            key("Esc"),
            Span::raw(" quit"),
        ]);
    }
    Line::from(vec![
        Span::raw(" "),
        key("type"),
        Span::raw(" search"),
        sep(),
        key("↑↓"),
        Span::raw(" move"),
        sep(),
        key("Enter"),
        Span::raw(" command"),
        sep(),
        key("Tab"),
        Span::raw(" review"),
        sep(),
        key("F1"),
        Span::raw(" help"),
        sep(),
        key("Esc"),
        Span::raw(" quit"),
    ])
}

/// A centred rectangle `pct_x` × `pct_y` percent of `area`, for the help popup.
fn centered_rect(pct_x: u16, pct_y: u16, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(vert[1])[1]
}

fn render_help(f: &mut Frame) {
    use ratatui::widgets::Clear;
    let area = centered_rect(70, 70, f.area());
    let row = |k: &'static str, desc: &'static str| {
        Line::from(vec![
            Span::styled(format!("  {k:<16}"), Style::default().fg(Color::Cyan)),
            Span::raw(desc),
        ])
    };
    let lines = vec![
        Line::from(""),
        row("type", "search your memory as you type"),
        row("↑ / ↓", "move up / down the results"),
        row("Ctrl-n / Ctrl-p", "move down / up (alternative)"),
        row("Backspace", "delete a search character"),
        row("Enter", "reveal the selected item's shell command"),
        row("Tab", "toggle review mode (approve/reject agent memories)"),
        row(
            "a / d",
            "in review mode: approve / reject the selected memory",
        ),
        row("F1", "toggle this help"),
        row("Esc / Ctrl-c", "quit"),
        Line::from(""),
        Line::styled(
            "  Browse here, then run the revealed command in your shell.",
            Style::default().fg(Color::DarkGray),
        ),
        Line::styled(
            "  Start it any time with:  mw tui",
            Style::default().fg(Color::DarkGray),
        ),
        Line::from(""),
        Line::styled(
            "  press Esc or F1 to close",
            Style::default().fg(Color::Yellow),
        ),
    ];
    let popup = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" MemoryWhale TUI — commands "),
    );
    f.render_widget(Clear, area); // blank whatever's behind the popup
    f.render_widget(popup, area);
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<(), String> {
    loop {
        terminal
            .draw(|f| render(app, f))
            .map_err(|e| format!("draw failed: {e}"))?;
        let Event::Key(key) = event::read().map_err(|e| format!("input failed: {e}"))? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // While the help overlay is open it's modal: Esc/F1/Enter close it, and
        // nothing else (so typed keys don't leak into the query behind it).
        if app.show_help {
            if matches!(key.code, KeyCode::Esc | KeyCode::F(1) | KeyCode::Enter)
                || (ctrl && key.code == KeyCode::Char('c'))
            {
                app.show_help = false;
            }
            continue;
        }

        // Keys shared by both modes.
        match key.code {
            KeyCode::F(1) => {
                app.show_help = true;
                continue;
            }
            KeyCode::Esc => return Ok(()),
            KeyCode::Char('c') if ctrl => return Ok(()),
            KeyCode::Tab => {
                app.toggle_review();
                continue;
            }
            _ => {}
        }

        // Review mode: approve/reject the selected pending memory.
        if app.review {
            match key.code {
                KeyCode::Down => app.move_review_sel(1),
                KeyCode::Up => app.move_review_sel(-1),
                KeyCode::Char('n') if ctrl => app.move_review_sel(1),
                KeyCode::Char('p') if ctrl => app.move_review_sel(-1),
                KeyCode::Char('a') => app.review_action(true),
                KeyCode::Char('d') => app.review_action(false),
                _ => {}
            }
            continue;
        }

        match key.code {
            KeyCode::Down => app.move_sel(1),
            KeyCode::Up => app.move_sel(-1),
            KeyCode::Char('n') if ctrl => app.move_sel(1),
            KeyCode::Char('p') if ctrl => app.move_sel(-1),
            KeyCode::Enter => app.reveal_action(),
            KeyCode::Backspace => {
                app.query.pop();
                app.recompute();
            }
            // Any other Ctrl-chord is ignored so it can't leak into the query.
            KeyCode::Char(c) if !ctrl => {
                app.query.push(c);
                app.recompute();
            }
            _ => {}
        }
    }
}

/// Entry point for `mw tui`.
pub fn run() -> Result<(), String> {
    let conn = Connection::open(crate::database_path()?)
        .map_err(|e| format!("failed to open memory db: {e}"))?;
    let _ = crate::migrate(&conn);
    let mems = load_memories(&conn).map_err(|e| format!("failed to load memories for TUI: {e}"))?;
    if mems.is_empty() {
        return Err("no memories yet — run some commands with `mw` first".to_string());
    }
    let mut app = App::new(BuiltinEngine::new(mems), conn);

    // ratatui::init() enters raw mode + the alternate screen and installs a
    // panic hook that restores the terminal; restore() undoes it on exit.
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use memorywhale_core::Memory;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn mem(id: i64, text: &str) -> Memory {
        Memory {
            id,
            text: text.into(),
            created_at: Utc::now(),
            last_used: Utc::now(),
            mentions: 1,
            importance: 0.5,
            tags: vec![],
            embedding: None,
            agent: None,
        }
    }

    /// An in-memory DB with the bookmarks schema migrated in. Optional pending
    /// rows are inserted as unapproved agent notes for the review-pane tests.
    fn conn_with_pending(pending: &[(&str, &str)]) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::migrate(&conn).unwrap();
        for (label, author) in pending {
            conn.execute(
                "INSERT INTO bookmarks (label, created_at, author_kind, author_name, source_session_id, approved)
                 VALUES (?1, '2026-07-12T09:00:00Z', 'agent', ?2, 41, 0)",
                rusqlite::params![label, author],
            )
            .unwrap();
        }
        conn
    }

    fn app_with(texts: &[(i64, &str)]) -> App {
        let mems = texts.iter().map(|(id, t)| mem(*id, t)).collect();
        App::new(BuiltinEngine::new(mems), conn_with_pending(&[]))
    }

    #[test]
    fn snippet_takes_first_nonempty_line_and_caps() {
        assert_eq!(snippet("\n\n  hello world  \nsecond", 100), "hello world");
        assert_eq!(snippet("abcdefgh", 3), "abc");
        assert_eq!(snippet("   ", 10), "");
    }

    #[test]
    fn empty_query_browses_all_typing_narrows() {
        let mut app = app_with(&[
            (1, "cargo build failed"),
            (2, "git push rejected"),
            (3, "cargo test flaky"),
        ]);
        assert_eq!(app.results.len(), 3, "empty query shows everything");

        app.query = "cargo".into();
        app.recompute();
        assert_eq!(
            app.results.len(),
            2,
            "narrows to memories containing the term"
        );
        assert!(app
            .results
            .iter()
            .all(|sm| sm.memory.text.contains("cargo")));

        app.query = "nonexistent".into();
        app.recompute();
        assert!(app.results.is_empty());
        assert_eq!(app.state.selected(), None);
    }

    #[test]
    fn agent_filter_is_applied_before_result_limit() {
        let mut memories = (1..=MAX_RESULTS as i64)
            .map(|id| {
                let mut memory = mem(id, "same unrelated memory");
                memory.agent = Some("claude".to_string());
                memory
            })
            .collect::<Vec<_>>();
        let mut rho_memory = mem(MAX_RESULTS as i64 + 1, "target rho memory");
        rho_memory.agent = Some("rho".to_string());
        memories.push(rho_memory);

        let mut app = App::new(BuiltinEngine::new(memories), conn_with_pending(&[]));
        app.query = "agent:rho".to_string();
        app.recompute();

        assert_eq!(app.results.len(), 1);
        assert_eq!(app.results[0].memory.agent.as_deref(), Some("rho"));
    }

    #[test]
    fn navigation_wraps_around() {
        let mut app = app_with(&[(1, "a"), (2, "b"), (3, "c")]);
        assert_eq!(app.state.selected(), Some(0));
        app.move_sel(-1);
        assert_eq!(
            app.state.selected(),
            Some(2),
            "up from the top wraps to the bottom"
        );
        app.move_sel(1);
        assert_eq!(
            app.state.selected(),
            Some(0),
            "down from the bottom wraps to the top"
        );
    }

    #[test]
    fn renders_without_panicking_and_shows_chrome() {
        let mut app = app_with(&[(1, "cargo build failed with E0308")]);
        let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
        terminal.draw(|f| render(&mut app, f)).unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("MemoryWhale"), "header present");
        assert!(text.contains("results"), "results count present");
        assert!(text.contains("cargo build failed"), "the memory is shown");
        // The key bar is always visible so the controls never disappear.
        assert!(text.contains("F1"), "key bar shows help hint");
        assert!(text.contains("quit"), "key bar shows how to exit");
    }

    #[test]
    fn help_overlay_lists_commands_and_how_to_exit() {
        let mut app = app_with(&[(1, "cargo build failed")]);
        app.show_help = true;
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|f| render(&mut app, f)).unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("commands"), "overlay titled with commands");
        assert!(text.contains("search your memory"), "explains search");
        assert!(text.contains("quit"), "explains how to exit");
        assert!(text.contains("mw tui"), "tells how to start it");
    }

    #[test]
    fn review_pane_shows_pending_provenance_and_ad_hints() {
        let conn = conn_with_pending(&[
            ("cache the tokenizer", "Claude Code"),
            ("retry on 429", "Codex"),
        ]);
        let mut app = App::new(BuiltinEngine::new(vec![mem(1, "seed")]), conn);
        app.review = true;
        app.refresh_pending();
        assert_eq!(app.pending.len(), 2);

        let mut terminal = Terminal::new(TestBackend::new(120, 24)).unwrap();
        terminal.draw(|f| render(&mut app, f)).unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("REVIEW MODE"), "review banner shown");
        assert!(
            text.contains("cache the tokenizer"),
            "pending item label shown"
        );
        assert!(
            text.contains("remembered by Claude Code"),
            "provenance shown"
        );
        assert!(text.contains("approve"), "a=approve hint shown");
        assert!(text.contains("reject"), "d=reject hint shown");
    }

    #[test]
    fn review_pane_empty_state_when_nothing_pending() {
        let mut app = app_with(&[(1, "seed")]); // no pending rows
        app.review = true;
        app.refresh_pending();
        assert!(app.pending.is_empty());

        let mut terminal = Terminal::new(TestBackend::new(120, 20)).unwrap();
        terminal.draw(|f| render(&mut app, f)).unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(
            text.contains("no memories awaiting review"),
            "empty state shown"
        );
    }

    #[test]
    fn approve_and_reject_drain_the_queue() {
        let conn = conn_with_pending(&[("keep me", "Claude Code"), ("drop me", "Codex")]);
        let mut app = App::new(BuiltinEngine::new(vec![mem(1, "seed")]), conn);
        app.review = true;
        app.refresh_pending();
        assert_eq!(app.pending.len(), 2);

        app.review_action(true); // approve the first
        assert!(app.status.starts_with("  approved #"));
        assert_eq!(app.pending.len(), 1, "approved item leaves the queue");

        app.review_action(false); // reject the last
        assert!(app.status.starts_with("  rejected #"));
        assert!(app.pending.is_empty(), "rejected item removed too");
    }

    fn buffer_text(buf: &ratatui::buffer::Buffer) -> String {
        buf.content().iter().map(|c| c.symbol()).collect()
    }
}
