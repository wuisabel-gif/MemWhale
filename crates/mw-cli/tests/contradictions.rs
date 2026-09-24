use std::process::{Command, Output};

fn mw(data_dir: &std::path::Path, bin: &str, args: &[&str]) -> Output {
    Command::new(bin)
        .args(args)
        .env("MEMORYWHALE_DATA_DIR", data_dir)
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    assert!(out.status.success(), "command failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn flag_is_created_listed_and_reviewed_through_cli() {
    let dir = std::env::temp_dir().join(format!("mw-contradictions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let remember = env!("CARGO_BIN_EXE_mw-remember");
    let bin = env!("CARGO_BIN_EXE_mw");
    for note in [
        "always use cargo build release",
        "do not use cargo build release \u{1b}]52;c;cHduZWQ=\u{7}\u{1b}[2J",
    ] {
        stdout(&mw(
            &dir,
            remember,
            &["--exit-code", "0", "--notes", note, "--", "true"],
        ));
    }

    let created = stdout(&mw(
        &dir,
        bin,
        &["contradictions", "1000000001", "1000000002"],
    ));
    assert!(created.contains("FLAG #1 pending"), "{created}");
    assert!(
        !created.contains('\u{1b}') && !created.contains('\u{7}'),
        "terminal controls from memory text must not reach the terminal: {created:?}"
    );

    let listed = stdout(&mw(&dir, bin, &["contradictions", "list"]));
    assert!(
        listed.contains("flag #1 pending memories #1000000001 vs #1000000002"),
        "{listed}"
    );

    let rejected = stdout(&mw(&dir, bin, &["contradictions", "reject", "1"]));
    assert!(rejected.contains("flag #1 rejected"), "{rejected}");
    let listed = stdout(&mw(&dir, bin, &["contradictions", "list"]));
    assert!(
        listed.contains("flag #1 rejected") && listed.contains("reviewed "),
        "{listed}"
    );

    let missing = mw(&dir, bin, &["contradictions", "confirm", "99"]);
    assert!(!missing.status.success());

    let help = stdout(&mw(&dir, bin, &["--help"]));
    assert!(help.contains("mw contradictions"), "{help}");
    let _ = std::fs::remove_dir_all(dir);
}
