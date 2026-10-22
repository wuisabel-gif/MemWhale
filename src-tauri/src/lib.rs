//! MemoryWhale desktop: a window onto the same local store the CLI and the
//! agent hooks write. `store` reads it; `tray` keeps watching from the menu bar.

use std::sync::Mutex;

mod store;
mod tray;

struct AppState {
    db: Mutex<rusqlite::Connection>,
}

pub fn run() {
    let conn = store::open().expect("failed to open the MemoryWhale database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            db: Mutex::new(conn),
        })
        .invoke_handler(tauri::generate_handler![
            store::overview,
            store::recall_history,
            store::list_runs,
            store::run_detail,
            store::search,
            store::list_lessons,
            store::add_lesson,
            store::approve_lesson,
            store::delete_lesson
        ])
        .setup(|app| {
            tray::install(app)?;
            tray::watch(app.handle().clone());
            Ok(())
        })
        // Closing the window keeps MemoryWhale in the menu bar, still watching.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building MemoryWhale")
        .run(|app, event| {
            // Clicking the Dock icon brings the hidden window back.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                tray::show(app);
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        });
}
