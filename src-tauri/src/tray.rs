//! The menu-bar icon, and a watcher that announces a recall note or warning
//! the moment a hook in another process writes one.

use crate::store::Event;
use crate::AppState;
use memorywhale_core::provenance;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

pub fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn install(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open MemoryWhale", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit MemoryWhale", true, None::<&str>)?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("MemoryWhale")
        .menu(&Menu::with_items(app, &[&open, &quit])?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// The newest recall hint id, and that event.
fn latest(app: &AppHandle) -> Option<(i64, Option<Event>)> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().ok()?;
    let id = conn
        .query_row("SELECT COALESCE(MAX(id), 0) FROM recall_hints", [], |r| {
            r.get(0)
        })
        .ok()?;
    let event = crate::store::events(&conn, 1)
        .ok()
        .and_then(|events| events.into_iter().next());
    Some((id, event))
}

/// Every few seconds, look for a note or warning a hook wrote since the last
/// look; notify, update the tray tooltip, and tell the window to refresh.
// ponytail: polls one MAX(id) every 5 s; watch the WAL file if that ever feels slow.
pub fn watch(app: AppHandle) {
    std::thread::spawn(move || {
        let mut seen = latest(&app).map_or(0, |(id, _)| id);
        loop {
            std::thread::sleep(Duration::from_secs(5));
            let Some((id, event)) = latest(&app) else {
                continue;
            };
            if id <= seen {
                continue;
            }
            seen = id;
            if let Some(event) = event {
                let (title, body) = announcement(&event);
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_tooltip(Some(format!("MemoryWhale: {title}")));
                }
                let _ = app.notification().builder().title(title).body(body).show();
            }
            let _ = app.emit("recall", id);
        }
    });
}

fn announcement(Event { event, stem }: &Event) -> (String, String) {
    let agent = match provenance::label(event.agent.as_deref()) {
        "terminal" => "you",
        agent => agent,
    };
    let program = if stem.is_empty() { "a command" } else { stem };
    let title = if event.kind == "warning" {
        format!("Warned {agent} before {program}")
    } else {
        format!("Left {agent} a note: {program} failed")
    };
    // Already redacted by store::events.
    (
        title,
        event.text.clone().unwrap_or_else(|| event.command.clone()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announcement_names_the_agent_and_program_and_prefers_the_exact_text() {
        let warning = Event {
            event: memorywhale_cli::RecallEvent {
                id: 1,
                kind: "warning".into(),
                at: "2026-10-21T10:00:00Z".into(),
                agent: Some("claude".into()),
                cwd: None,
                command: "L=/tmp/x cargo build --release".into(),
                text: Some("seen 2 times; confirmed fix: xcode-select --install".into()),
                outcome: "no pass yet".into(),
            },
            stem: "cargo build".into(),
        };
        let (title, body) = announcement(&warning);
        assert_eq!(title, "Warned claude before cargo build");
        assert_eq!(body, "seen 2 times; confirmed fix: xcode-select --install");

        let mut note = warning;
        note.event.kind = "note".into();
        note.event.agent = None;
        note.event.text = None;
        note.stem = String::new();
        assert_eq!(
            announcement(&note),
            (
                "Left you a note: a command failed".into(),
                "L=/tmp/x cargo build --release".into()
            )
        );
    }
}
