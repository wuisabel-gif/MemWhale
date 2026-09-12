//! Opt-in local instruction delivery (Interfaces only); never launches a client.
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

const LIMIT: u64 = 64 * 1024;
const MARKER: &str = ".memorywhale-owned.json";
const USAGE: &str = "usage: mw integrate rho|codex|cursor --skill <local SKILL.md> [--skills-dir <directory>] [--revert | --check | --dry-run]; skill-only: MCP and capture configuration are unchanged";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    name: String,
    description: String,
    license: Option<String>,
    compatibility: Option<String>,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
    #[serde(rename = "disable-model-invocation")]
    manual_only: Option<bool>,
}

fn validate(text: &str, client: &str) -> Result<Metadata, String> {
    let normalized = text.replace("\r\n", "\n");
    let rest = normalized
        .strip_prefix("---\n")
        .ok_or("SKILL.md needs YAML frontmatter")?;
    let (header, body) = rest
        .split_once("\n---\n")
        .ok_or("SKILL.md needs closing frontmatter and a body")?;
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(header).map_err(|_| "invalid skill metadata")?;
    if !yaml.get("name").is_some_and(serde_yaml::Value::is_string)
        || !yaml
            .get("description")
            .is_some_and(serde_yaml::Value::is_string)
    {
        return Err("skill name and description must be YAML strings".into());
    }
    for field in ["license", "compatibility"] {
        if yaml.get(field).is_some_and(|value| !value.is_string()) {
            return Err("license and compatibility must be strings".into());
        }
    }
    if let Some(value) = yaml.get("metadata") {
        let map = value.as_mapping().ok_or("metadata must be a string map")?;
        if map
            .iter()
            .any(|(key, value)| !key.is_string() || !value.is_string())
        {
            return Err("metadata must be a string map".into());
        }
    }
    if let Some(value) = yaml.get("disable-model-invocation") {
        if !value.is_bool() || client == "codex" {
            return Err("disable-model-invocation requires a boolean and a supported client (Rho or Cursor); it is not silently discarded for Codex".into());
        }
    }
    let meta: Metadata = serde_yaml::from_str(header).map_err(|_| {
        "unsupported skill metadata; use name, description, optional license/compatibility/metadata, and supported invocation control; tool permissions are not imported"
    })?;
    if meta
        .license
        .as_ref()
        .is_some_and(|value| value.trim().is_empty())
        || meta
            .compatibility
            .as_ref()
            .is_some_and(|value| value.trim().is_empty() || value.chars().count() > 500)
        || meta.metadata.len() > 64
    {
        return Err("invalid or oversized optional skill metadata".into());
    }
    let name = &meta.name;
    if name.is_empty()
        || name.len() > 64
        || name.starts_with('-')
        || name.ends_with('-')
        || name.contains("--")
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || name == "memorywhale"
    {
        return Err(
            "skill name must be a safe lowercase slug (1–64 bytes); memorywhale is reserved".into(),
        );
    }
    if meta.description.trim().is_empty()
        || meta.description.chars().count() > 1024
        || body.trim().is_empty()
        || text.contains('\0')
    {
        return Err("skill requires a nonempty description (at most 1024 characters) and body, without NUL bytes".into());
    }
    Ok(meta)
}

// Do not canonicalize an input and silently follow a link. Check each existing
// component, including dangling links, before touching the requested location.
fn safe_path(path: &Path) -> Result<PathBuf, String> {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("parent traversal is not allowed".into());
    }
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| "cannot resolve working directory")?
            .join(path)
    };
    let mut current = PathBuf::new();
    for part in path.components() {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err("symlinks are not allowed in skill paths".into())
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err("cannot inspect skill path".into()),
        }
    }
    Ok(path)
}

fn read(path: &Path, limit: u64) -> Result<String, String> {
    safe_path(path)?;
    if !fs::symlink_metadata(path)
        .map_err(|_| "cannot inspect skill or ownership file")?
        .is_file()
    {
        return Err("skill and ownership files must be regular files".into());
    }
    let file = fs::File::open(path).map_err(|_| "cannot open skill or ownership file")?;
    if !file
        .metadata()
        .map_err(|_| "cannot inspect skill file")?
        .is_file()
    {
        return Err("skill and ownership files must be regular files".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read skill file")?;
    if bytes.len() as u64 > limit {
        return Err("skill or ownership file exceeds size limit".into());
    }
    String::from_utf8(bytes).map_err(|_| "skill file must be UTF-8".into())
}

fn create(path: &Path, content: &str) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|_| "cannot create skill file; no existing file was overwritten")?;
    file.write_all(content.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|_| "cannot persist skill file".to_string())
}

/// Serialize cooperating installers outside the directory that revert removes.
struct SkillLock(PathBuf);

impl SkillLock {
    fn acquire(path: &Path) -> Result<Self, String> {
        safe_path(path)?;
        create(path, "MemoryWhale skill operation in progress\n").map_err(|_| {
            "skill operation is locked; inspect any interrupted operation before retrying"
        })?;
        Ok(Self(path.to_path_buf()))
    }
}

impl Drop for SkillLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn validate_owned(target: &Path, marker: &Path, source: &str) -> Result<(), String> {
    let owned: String = serde_json::from_str(&read(marker, LIMIT * 6 + 2)?)
        .map_err(|_| "invalid ownership record")?;
    if owned != source || read(target, LIMIT)? != owned {
        return Err("skill differs from the owned snapshot or supplied source; refusing to overwrite or remove user changes".into());
    }
    Ok(())
}

pub fn cli(args: &[String]) -> Result<(), String> {
    let client = args.first().map(String::as_str).ok_or(USAGE)?;
    if !matches!(client, "rho" | "codex" | "cursor") {
        return Err(USAGE.into());
    }
    let (mut source, mut root) = (None, None);
    let mut mode = "install";
    let mut iter = args[1..].iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--skill" if source.is_none() => {
                source = Some(PathBuf::from(iter.next().ok_or(USAGE)?))
            }
            "--skills-dir" if root.is_none() => {
                root = Some(PathBuf::from(iter.next().ok_or(USAGE)?))
            }
            "--revert" | "--check" | "--dry-run" if mode == "install" => mode = arg.as_str(),
            _ => return Err(USAGE.into()),
        }
    }
    let source = source.ok_or(USAGE)?;
    if source.file_name().and_then(|s| s.to_str()) != Some("SKILL.md") {
        return Err("source must be a local SKILL.md file".into());
    }
    let text = read(&source, LIMIT)?;
    let meta = validate(&text, client)?;
    let custom = root.is_some();
    let root = safe_path(&match root {
        Some(root) => root,
        None => {
            let home = dirs::home_dir().ok_or("cannot resolve home directory")?;
            match client {
                "rho" => {
                    let default_home = home.join(".rho");
                    if std::env::var_os("RHO_HOME")
                        .filter(|path| !path.is_empty())
                        .is_some_and(|path| Path::new(&path) != default_home.as_path())
                    {
                        return Err("Rho 2.9.1 discovers loose skills from HOME/project roots, not a custom RHO_HOME; specify --skills-dir explicitly (for example .agents/skills in your project)".into());
                    }
                    default_home.join("skills")
                }
                "codex" => home.join(".agents/skills"),
                _ => home.join(".cursor/skills"),
            }
        }
    })?;
    let dir = safe_path(&root.join(&meta.name))?;
    let target = safe_path(&dir.join("SKILL.md"))?;
    let marker = safe_path(&dir.join(MARKER))?;
    let lock_path = safe_path(&root.join(format!(".{}.memorywhale-skill.lock", meta.name)))?;
    let mutating = matches!(mode, "install" | "--revert");
    let _lock = if mutating
        && (mode == "install"
            || root
                .try_exists()
                .map_err(|_| "cannot inspect skills root")?)
    {
        fs::create_dir_all(&root).map_err(|_| "cannot create skills root")?;
        Some(SkillLock::acquire(&lock_path)?)
    } else {
        if lock_path
            .try_exists()
            .map_err(|_| "cannot inspect skill lock")?
        {
            return Err("skill operation is locked; local verification must wait".into());
        }
        None
    };
    let exists = dir.try_exists().map_err(|_| "cannot inspect destination")?;
    if exists {
        validate_owned(&target, &marker, &text)?;
    }
    match mode {
        "--check" if !exists => return Err("skill is not installed".into()),
        "--revert" if exists => {
            super::skill_files::remove(&dir, &text)?;
        }
        "install" if !exists => {
            super::skill_files::install(&dir, &text)?;
        }
        _ => (),
    }
    println!(
        "{client}: local skill {}: {mode}{}",
        meta.name,
        if exists {
            " (existing owned skill)"
        } else {
            ""
        }
    );
    println!("Portable metadata validated; MCP/capture unchanged. Client runtime discovery and invocation NOT verified.");
    if meta.manual_only == Some(true) {
        println!("This skill requests manual invocation only; use a client version that honors disable-model-invocation.");
    }
    println!("Destination: {}", target.display());
    if custom {
        println!("Custom skills root: the client must discover this directory; no discovery settings were changed.");
    }
    Ok(())
}

#[cfg(test)]
#[path = "portable_tests.rs"]
mod tests;
