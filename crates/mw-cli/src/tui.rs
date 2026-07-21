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

use mw_memory::engine::{BuiltinEngine, MemoryEngine};
use mw_memory::sqlite::{decode_id, load_memories, Source};
use mw_memory::{Query, ScoredMemory};

/// How many results to hold in the list — plenty for browsing, cheap to rank.
const MAX_RESULTS: usize = 500;

struct App {
    engine: BuiltinEngine,
    now: DateTime<Utc>,
    query: String,
    results: Vec<ScoredMemory>,
    state: ListState,
    /// The last "reveal" action (the shell command for the selected memory).
    status: String,
    /// Whether the F1 help overlay is open.
    show_help: bool,
}

impl App {
    fn new(engine: BuiltinEngine) -> Self {
        let mut app = App {
            engine,
            now: Utc::now(),
            query: String::new(),
            results: Vec::new(),
            state: ListState::default(),
            status: String::new(),
            show_help: false,
        };
        app.recompute();
        app
    }

    /// Re-rank against the current query. Empty query ranks by recency/importance
    /// (a "recent memory" browse). A non-empty query both *reranks* by relevance
    /// and *narrows* to memories that actually contain a query term — otherwise
    /// the engine returns everything (just reordered), which reads wrong for a
    /// search box.
    fn recompute(&mut self) {
        let q = Query::new(&self.query, self.now);
        let mut hits = self.engine.retrieve(&q, MAX_RESULTS);
        let terms: Vec<String> =
            self.query.to_lowercase().split_whitespace().map(String::from).collect();
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
        self.state.select(Some((cur + delta).rem_euclid(len) as usize));
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

    // Search box.
    let search = Paragraph::new(Line::from(vec![
        Span::styled("search ", Style::default().fg(Color::Cyan)),
        Span::raw(&app.query),
        Span::styled("▏", Style::default().fg(Color::Cyan)), // cursor
    ]))
    .block(Block::default().borders(Borders::ALL).title(" MemoryWhale "));
    f.render_widget(search, rows[0]);

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
                Span::styled(format!("{:>3}% ", sm.percent()), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<8} ", source.tag()), Style::default().fg(Color::DarkGray)),
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
        None => Paragraph::new("(no matches — try a different search, or capture some commands first)"),
        Some(sm) => {
            let (source, id) = decode_id(sm.memory.id);
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(format!("[{}] ", source.tag()), Style::default().fg(Color::Yellow)),
                    Span::styled(format!("#{id}  "), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{}% match", sm.percent()), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(""),
            ];
            for line in sm.memory.text.lines() {
                lines.push(Line::raw(line.to_string()));
            }
            let reasons = sm.reasons();
            if !reasons.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::styled("why this ranked here:", Style::default().fg(Color::DarkGray)));
                for r in reasons {
                    lines.push(Line::styled(format!("  • {r}"), Style::default().fg(Color::DarkGray)));
                }
            }
            Paragraph::new(lines).wrap(Wrap { trim: false })
        }
    };
    f.render_widget(detail.block(Block::default().borders(Borders::ALL).title(" detail ")), body[1]);

    // Status line: the revealed action, or a gentle nudge toward it.
    let status = if app.status.is_empty() {
        Line::styled(
            "  press Enter on a result to get its shell command",
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Line::styled(app.status.clone(), Style::default().fg(Color::Green))
    };
    f.render_widget(Paragraph::new(status), rows[2]);

    // Key bar: always visible, so the controls never disappear.
    f.render_widget(Paragraph::new(key_bar()), rows[3]);

    // F1 help overlay, drawn last so it sits on top.
    if app.show_help {
        render_help(f);
    }
}

/// The always-on key hints. Full list lives in the F1 overlay.
fn key_bar() -> Line<'static> {
    let key = |k: &'static str| Span::styled(k, Style::default().fg(Color::Cyan));
    let sep = || Span::styled("  ·  ", Style::default().fg(Color::DarkGray));
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
        Line::styled("  press Esc or F1 to close", Style::default().fg(Color::Yellow)),
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
        terminal.draw(|f| render(app, f)).map_err(|e| format!("draw failed: {e}"))?;
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

        match key.code {
            KeyCode::F(1) => app.show_help = true,
            KeyCode::Esc => return Ok(()),
            KeyCode::Char('c') if ctrl => return Ok(()),
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
    let conn =
        Connection::open(crate::database_path()?).map_err(|e| format!("failed to open memory db: {e}"))?;
    let _ = crate::migrate(&conn);
    let mems = load_memories(&conn);
    if mems.is_empty() {
        return Err("no memories yet — run some commands with `mw` first".to_string());
    }
    let mut app = App::new(BuiltinEngine::new(mems));

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
    use mw_memory::Memory;
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
        }
    }

    fn app_with(texts: &[(i64, &str)]) -> App {
        let mems = texts.iter().map(|(id, t)| mem(*id, t)).collect();
        App::new(BuiltinEngine::new(mems))
    }

    #[test]
    fn snippet_takes_first_nonempty_line_and_caps() {
        assert_eq!(snippet("\n\n  hello world  \nsecond", 100), "hello world");
        assert_eq!(snippet("abcdefgh", 3), "abc");
        assert_eq!(snippet("   ", 10), "");
    }

    #[test]
    fn empty_query_browses_all_typing_narrows() {
        let mut app = app_with(&[(1, "cargo build failed"), (2, "git push rejected"), (3, "cargo test flaky")]);
        assert_eq!(app.results.len(), 3, "empty query shows everything");

        app.query = "cargo".into();
        app.recompute();
        assert_eq!(app.results.len(), 2, "narrows to memories containing the term");
        assert!(app.results.iter().all(|sm| sm.memory.text.contains("cargo")));

        app.query = "nonexistent".into();
        app.recompute();
        assert!(app.results.is_empty());
        assert_eq!(app.state.selected(), None);
    }

    #[test]
    fn navigation_wraps_around() {
        let mut app = app_with(&[(1, "a"), (2, "b"), (3, "c")]);
        assert_eq!(app.state.selected(), Some(0));
        app.move_sel(-1);
        assert_eq!(app.state.selected(), Some(2), "up from the top wraps to the bottom");
        app.move_sel(1);
        assert_eq!(app.state.selected(), Some(0), "down from the bottom wraps to the top");
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

    fn buffer_text(buf: &ratatui::buffer::Buffer) -> String {
        buf.content().iter().map(|c| c.symbol()).collect()
    }
}
