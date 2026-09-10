use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

const SKILL: &str = "---\nname: concise-response\ndescription: Give concise responses when requested\n---\nUse short English summaries unless the user requests another language.\n";

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mw-portable-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap()
        ));
        fs::create_dir(&path).unwrap();
        Self(fs::canonicalize(path).unwrap())
    }
    fn run(&self, client: &str, flags: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_mw"))
            .args(["integrate", client, "--skill"])
            .arg(self.0.join("SKILL.md"))
            .args(["--skills-dir"])
            .arg(self.0.join("skills"))
            .args(flags)
            .env("HOME", &self.0)
            .env("RHO_HOME", self.0.join("rho"))
            .env("MEMORYWHALE_DATA_DIR", self.0.join("data"))
            .env("PATH", "")
            .output()
            .unwrap()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn default_client_roots_are_resolved_in_the_sandbox() {
    for (client, relative) in [
        ("rho", ".rho/skills"),
        ("codex", ".agents/skills"),
        ("cursor", ".cursor/skills"),
    ] {
        let s = Sandbox::new();
        fs::write(s.0.join("SKILL.md"), SKILL).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_mw"))
            .args(["integrate", client, "--skill"])
            .arg(s.0.join("SKILL.md"))
            .env("HOME", &s.0)
            .env("USERPROFILE", &s.0)
            .env("RHO_HOME", s.0.join(".rho"))
            .env("MEMORYWHALE_DATA_DIR", s.0.join("data"))
            .env("PATH", "")
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
        assert!(s
            .0
            .join(relative)
            .join("concise-response/SKILL.md")
            .is_file());
    }
}

#[test]
fn custom_rho_home_requires_an_explicit_discoverable_skill_root() {
    let s = Sandbox::new();
    fs::write(s.0.join("SKILL.md"), SKILL).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(["integrate", "rho", "--skill"])
        .arg(s.0.join("SKILL.md"))
        .env("HOME", &s.0)
        .env("USERPROFILE", &s.0)
        .env("RHO_HOME", s.0.join("profile"))
        .env("MEMORYWHALE_DATA_DIR", s.0.join("data"))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("--skills-dir"));
    assert!(!s.0.join(".rho").exists());
    assert!(!s.0.join("profile").exists());
    assert!(s.run("rho", &[]).status.success());
}

#[test]
fn portable_clients_install_check_and_revert_only_owned_skill() {
    for client in ["rho", "codex", "cursor"] {
        let s = Sandbox::new();
        fs::write(s.0.join("SKILL.md"), SKILL).unwrap();
        assert!(!s.run(client, &["--check"]).status.success());
        assert!(s.run(client, &["--dry-run"]).status.success());
        assert!(!s.0.join("skills").exists());
        for _ in 0..2 {
            let out = s.run(client, &[]);
            assert!(out.status.success(), "{out:?}");
        }
        let dir = s.0.join("skills/concise-response");
        assert_eq!(fs::read_to_string(dir.join("SKILL.md")).unwrap(), SKILL);
        assert!(s.run(client, &["--check"]).status.success());
        fs::write(dir.join("user.txt"), "keep").unwrap();
        assert!(s.run(client, &["--revert"]).status.success());
        assert!(dir.join("user.txt").exists());
        assert!(!dir.join("SKILL.md").exists());
        assert!(!s.0.join("rho").exists());
    }
}

#[test]
fn modified_or_unowned_skills_are_never_adopted_or_deleted() {
    let s = Sandbox::new();
    fs::write(s.0.join("SKILL.md"), SKILL).unwrap();
    assert!(s.run("rho", &[]).status.success());
    let target = s.0.join("skills/concise-response/SKILL.md");
    fs::write(&target, "user content").unwrap();
    for flag in [vec![], vec!["--revert"], vec!["--check"], vec!["--dry-run"]] {
        assert!(!s.run("rho", &flag).status.success());
        assert_eq!(fs::read_to_string(&target).unwrap(), "user content");
    }
    fs::remove_file(s.0.join("skills/concise-response/.memorywhale-owned.json")).unwrap();
    fs::write(&target, SKILL).unwrap();
    assert!(!s.run("codex", &[]).status.success());
}

#[test]
fn invalid_metadata_and_unsupported_flags_do_not_write() {
    let s = Sandbox::new();
    for text in [
        SKILL.replace("concise-response", "../escape"),
        SKILL.replace("concise-response", "memorywhale"),
        SKILL.replace("concise-response", "123"),
        SKILL.replace("description:", "tools: bash\ndescription:"),
        "---\nname: x\ndescription: ''\n---\nbody".into(),
        "x".repeat(65537),
    ] {
        fs::write(s.0.join("SKILL.md"), text).unwrap();
        assert!(!s.run("rho", &[]).status.success());
        assert!(!s.0.join("skills").exists());
    }
    fs::write(s.0.join("SKILL.md"), [0xff]).unwrap();
    assert!(!s.run("rho", &[]).status.success());
    fs::write(s.0.join("SKILL.md"), SKILL).unwrap();
    for flags in [
        vec!["--http"],
        vec!["--token", "do-not-print-secret"],
        vec!["--check", "--revert"],
    ] {
        let out = s.run("rho", &flags);
        assert!(!out.status.success());
        assert!(!String::from_utf8_lossy(&out.stderr).contains("do-not-print-secret"));
    }
}

#[test]
fn standard_metadata_is_preserved_and_manual_invocation_is_client_specific() {
    let text = SKILL.replace(
        "description:",
        "license: MIT\ncompatibility: A local coding client\nmetadata:\n  category: productivity\ndescription:",
    );
    for client in ["rho", "codex", "cursor"] {
        let s = Sandbox::new();
        fs::write(s.0.join("SKILL.md"), &text).unwrap();
        let result = s.run(client, &[]);
        assert!(result.status.success(), "{result:?}");
        assert_eq!(
            fs::read_to_string(s.0.join("skills/concise-response/SKILL.md")).unwrap(),
            text
        );
    }
    let manual = text.replace(
        "license: MIT",
        "license: MIT\ndisable-model-invocation: true",
    );
    for client in ["rho", "cursor", "codex"] {
        let s = Sandbox::new();
        fs::write(s.0.join("SKILL.md"), &manual).unwrap();
        let result = s.run(client, &[]);
        assert_eq!(result.status.success(), client != "codex", "{result:?}");
        if client == "codex" {
            assert!(!s.0.join("skills").exists());
        }
    }
}

#[test]
fn skill_metadata_cannot_import_tool_permissions_or_coerce_types() {
    let s = Sandbox::new();
    for field in [
        "allowed-tools: Bash",
        "disable-model-invocation: 'true'",
        "license: 42",
        "metadata: {flag: true}",
        "metadata: {1: value}",
        "hooks: {}",
    ] {
        fs::write(
            s.0.join("SKILL.md"),
            SKILL.replace("description:", &format!("{field}\ndescription:")),
        )
        .unwrap();
        assert!(!s.run("rho", &[]).status.success());
        assert!(!s.0.join("skills").exists());
    }
}

#[cfg(unix)]
#[test]
fn refuses_source_and_destination_symlinks() {
    use std::os::unix::fs::symlink;
    let s = Sandbox::new();
    fs::write(s.0.join("real"), SKILL).unwrap();
    symlink(s.0.join("real"), s.0.join("SKILL.md")).unwrap();
    assert!(!s.run("rho", &[]).status.success());
    fs::remove_file(s.0.join("SKILL.md")).unwrap();
    fs::write(s.0.join("SKILL.md"), SKILL).unwrap();
    fs::create_dir(s.0.join("outside")).unwrap();
    symlink(s.0.join("outside"), s.0.join("skills")).unwrap();
    assert!(!s.run("cursor", &[]).status.success());
    assert_eq!(fs::read_dir(s.0.join("outside")).unwrap().count(), 0);
}
