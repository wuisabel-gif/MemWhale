//! Real CLI search, with a dedicated synthetic store and positive control.
use std::process::Command;

#[test]
fn unrelated_query_abstains_while_a_recorded_match_is_returned() {
    let dir = std::env::temp_dir().join(format!("mw-abstention-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let conn = memorywhale_cli::storage::open_path(&dir.join("memorywhale.sqlite3")).unwrap();
    conn.execute("INSERT INTO bookmarks (label, created_at, approved) VALUES ('synthetic quasar build repaired', '2026-11-01T00:00:00Z', 1)", []).unwrap();
    drop(conn);
    let search = |query| {
        let output = Command::new(env!("CARGO_BIN_EXE_mw"))
            .args(["search", query])
            .env("MEMORYWHALE_DATA_DIR", &dir)
            .env("HOME", &dir)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let positive = search("quasar");
    assert!(positive.contains("synthetic quasar"), "{positive}");
    let negative = search("xylophonium");
    assert!(negative.contains("Insufficient evidence"), "{negative}");
    assert!(!negative.contains("synthetic quasar"), "{negative}");
    std::fs::remove_dir_all(dir).unwrap();
}
