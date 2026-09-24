//! `mw search --ranking` end to end: option/value parsing and MemPalace routing.

use std::path::PathBuf;
use std::process::{Command, Output};

fn sandbox() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "mw-search-ranking-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    for dir in ["data", "home"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    root
}

fn run(root: &PathBuf, bin: &str, args: &[&str]) -> Output {
    Command::new(bin)
        .args(args)
        .env_clear()
        .env("HOME", root.join("home"))
        .env("MEMORYWHALE_DATA_DIR", root.join("data"))
        .env("XDG_DATA_HOME", root.join("data"))
        .current_dir(root)
        .output()
        .unwrap()
}

fn text(out: &Output) -> (String, String) {
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn ranking_option_parses_its_value_and_bypasses_mempalace() {
    let root = sandbox();
    let mw = env!("CARGO_BIN_EXE_mw");
    let remember = run(
        &root,
        env!("CARGO_BIN_EXE_mw-remember"),
        &["--", "echo bayesian-marker"],
    );
    assert!(remember.status.success(), "{remember:?}");

    // A plain `bayesian` word is a query term, not a ranking flag.
    let out = run(&root, mw, &["search", "bayesian"]);
    let (stdout, _) = text(&out);
    assert!(out.status.success(), "{out:?}");
    assert!(
        stdout.contains("matches for \"bayesian\"  (ranked)"),
        "{stdout}"
    );

    // `--ranking default` consumes `default`; it is not searched for.
    let out = run(&root, mw, &["search", "bayesian", "--ranking", "default"]);
    let (stdout, _) = text(&out);
    assert!(
        stdout.contains("matches for \"bayesian\"  (ranked)"),
        "{stdout}"
    );

    // Bare or unknown values are usage errors.
    for bad in [
        &["search", "x", "--ranking"][..],
        &["search", "x", "--ranking", "bayes"],
    ] {
        let out = run(&root, mw, bad);
        assert!(!out.status.success(), "{bad:?} {out:?}");
    }

    // With MemPalace configured (and unreachable), an explicit ranking must use
    // the builtin scorer instead of silently taking MemPalace's scores.
    std::fs::write(
        root.join("data/config.toml"),
        "engine = \"mempalace\"\nmempalace_command = \"/nonexistent/mempalace-mcp\"\n",
    )
    .unwrap();
    let out = run(
        &root,
        mw,
        &["search", "bayesian", "--ranking", "bayesian", "--explain"],
    );
    let (stdout, stderr) = text(&out);
    assert!(out.status.success(), "{out:?}");
    assert!(!stderr.contains("mempalace"), "{stderr}");
    assert!(stdout.contains("ranked by posterior_proxy"), "{stdout}");
    assert!(stdout.contains("= sigmoid("), "{stdout}");
    // Without an explicit ranking, MemPalace is still tried first.
    let out = run(&root, mw, &["search", "bayesian"]);
    let (_, stderr) = text(&out);
    assert!(stderr.contains("mempalace unavailable"), "{stderr}");

    let _ = std::fs::remove_dir_all(&root);
}
