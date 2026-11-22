//! `mw knowledge` end to end, through the binary, against a throwaway home.

use std::path::Path;
use std::process::Command;

fn mw(home: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_mw"))
        .env("HOME", home)
        .env("MEMORYWHALE_DATA_DIR", home.join("data"))
        .args(args)
        .output()
        .expect("run mw");
    (
        out.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

#[test]
fn one_command_imports_then_review_decides_what_a_client_sees() {
    let home = std::env::temp_dir().join(format!("mw-knowledge-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(home.join(".claude/projects/-w-alpha/memory")).unwrap();
    std::fs::create_dir_all(home.join("Downloads/chat-export")).unwrap();
    std::fs::write(
        home.join(".claude/projects/-w-alpha/a.jsonl"),
        r#"{"type":"user","sessionId":"s1","cwd":"/w/alpha","message":{"role":"user","content":"I prefer small pull requests."}}"#,
    )
    .unwrap();
    std::fs::write(
        home.join(".claude/projects/-w-alpha/memory/quotes.md"),
        "---\nname: quotes\ndescription: \"Use \\\"double\\\" quotes in docs\"\nmetadata:\n  type: feedback\n---\n",
    )
    .unwrap();
    std::fs::write(
        home.join("Downloads/chat-export/conversations.json"),
        r#"[{"conversation_id":"g1","mapping":{"a":{"message":{"author":{"role":"user"},"content":{"parts":["I work at a robotics lab."]}}}}}]"#,
    )
    .unwrap();

    // No arguments: every source under home.
    let (ok, out) = mw(&home, &["knowledge", "import"]);
    assert!(ok, "{out}");
    assert!(
        out.contains("3 new statement(s) are waiting for review"),
        "{out}"
    );
    assert!(!out.contains("No ChatGPT or Claude chat export"), "{out}");

    // Nothing is shown to a client before review.
    let (_, out) = mw(&home, &["knowledge", "search"]);
    assert_eq!(out, "");
    let (_, pending) = mw(&home, &["knowledge", "pending"]);
    assert!(
        pending.contains("Use \"double\" quotes in docs"),
        "{pending}"
    );
    let id_of = |text: &str| -> String {
        let at = pending.find(text).unwrap();
        let line = pending[..at].lines().rev().nth(1).unwrap();
        line.trim_start_matches('#')
            .split_whitespace()
            .next()
            .unwrap()
            .to_string()
    };
    let (ok, _) = mw(
        &home,
        &["knowledge", "approve", &id_of("small pull requests")],
    );
    assert!(ok);
    let (ok, _) = mw(&home, &["knowledge", "keep", &id_of("robotics lab")]);
    assert!(ok);
    let (ok, out) = mw(&home, &["knowledge", "reject", "all"]);
    assert!(ok && out.contains("1 statement(s)"), "{out}");

    // The approved statement is exported; the private one only on request.
    let (_, pack) = mw(&home, &["knowledge", "export", "--client", "codex"]);
    assert!(
        pack.contains("small pull requests") && !pack.contains("robotics"),
        "{pack}"
    );
    assert!(
        pack.starts_with("<!-- memorywhale:knowledge:start"),
        "{pack}"
    );
    let (_, pack) = mw(&home, &["knowledge", "export", "--private"]);
    assert!(pack.contains("robotics"), "{pack}");

    // Written into an instruction file: my own lines stay, and a second
    // write replaces the block instead of adding another.
    let rules = home.join("AGENTS.md");
    std::fs::write(&rules, "# Mine\n\nKeep this line.\n").unwrap();
    for _ in 0..2 {
        let (ok, out) = mw(
            &home,
            &["knowledge", "export", "--into", rules.to_str().unwrap()],
        );
        assert!(ok, "{out}");
    }
    let written = std::fs::read_to_string(&rules).unwrap();
    assert!(
        written.starts_with("# Mine\n\nKeep this line.\n"),
        "{written}"
    );
    assert_eq!(
        written.matches("memorywhale:knowledge:start").count(),
        1,
        "{written}"
    );
    assert!(written.contains("small pull requests"), "{written}");

    // Importing again proposes nothing, the rejected statement included.
    let (_, out) = mw(&home, &["knowledge", "import"]);
    assert!(out.contains("0 new statement(s)"), "{out}");

    let (ok, out) = mw(&home, &["knowledge"]);
    assert!(!ok && out.contains("usage: mw knowledge import"), "{out}");
    let (_, out) = mw(&home, &["knowledge", "doctor"]);
    assert!(out.contains("not a security boundary"), "{out}");
    let _ = std::fs::remove_dir_all(&home);
}
