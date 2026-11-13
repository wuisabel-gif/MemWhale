//! Lessons that travel with a repository: `mw memory publish` writes them to
//! `.memorywhale/lessons.md`; `mw memory adopt` reads that file on another
//! store, into the review queue only.

use std::path::{Path, PathBuf};
use std::process::Command;

fn dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-repo-lessons-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn mw(store: &Path, cwd: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_mw"))
        .env("MEMORYWHALE_DATA_DIR", store)
        .env_remove("CLAUDECODE")
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run mw");
    assert!(out.status.success(), "mw {args:?} failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn lessons_are_published_then_adopted_into_review_only() {
    let (repo, elsewhere) = (dir("repo"), dir("elsewhere"));
    let (mine, theirs) = (dir("store-a"), dir("store-b"));
    assert!(Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());

    // Only a lesson saved inside the repository is published.
    mw(
        &mine,
        &repo,
        &[
            "remember",
            "linker cc not found -> run: xcode-select --install",
        ],
    );
    mw(
        &mine,
        &elsewhere,
        &["remember", "unrelated lesson from another project"],
    );
    let out = mw(&mine, &repo, &["memory", "publish"]);
    assert!(out.contains("wrote 1 lesson"), "{out}");
    let file = repo.join(".memorywhale/lessons.md");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains("- linker cc not found -> run: xcode-select --install"),
        "{text}"
    );
    assert!(!text.contains("unrelated lesson"), "{text}");
    assert!(mw(&mine, &repo, &["memory", "publish"]).contains("no new lessons"));

    // A teammate's line, and one with terminal control bytes, arrive by git.
    std::fs::write(
        &file,
        format!("{text}- run npm ci after switching branches\n- \u{1b}[31mred\u{1b}[0m lesson\n"),
    )
    .unwrap();
    // Publishing again keeps lines this store did not write.
    mw(
        &mine,
        &repo,
        &["remember", "--force", "use cargo build --locked in CI"],
    );
    mw(&mine, &repo, &["memory", "publish"]);
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains("- run npm ci after switching branches"),
        "{text}"
    );
    assert!(text.contains("- use cargo build --locked in CI"), "{text}");

    // Another store adopts: everything lands in the review queue, not in use.
    let out = mw(&theirs, &repo, &["memory", "adopt"]);
    assert!(out.contains("4 new lesson(s)"), "{out}");
    let pending = mw(&theirs, &repo, &["memory", "pending"]);
    assert!(pending.contains("by repo"), "{pending}");
    assert!(
        pending.contains("red lesson") && !pending.contains('\u{1b}'),
        "{pending}"
    );
    let search = mw(&theirs, &repo, &["search", "linker", "cc", "not", "found"]);
    assert!(search.contains("Insufficient evidence"), "{search}");
    assert!(mw(&theirs, &repo, &["memory", "adopt"]).contains("0 new lesson(s)"));

    // Approved, it is used like any lesson.
    mw(&theirs, &repo, &["memory", "approve", "1"]);
    let search = mw(&theirs, &repo, &["search", "linker", "cc", "not", "found"]);
    assert!(search.contains("xcode-select --install"), "{search}");
}
