//! Local Git repository and worktree identity discovery.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryIdentity {
    pub id: String,
    pub name: String,
    pub worktree_root: String,
}

/// Resolve the repository containing `cwd` without invoking Git or contacting a
/// remote. Malformed worktree pointers are ignored rather than guessed at.
pub fn discover(cwd: &str) -> Option<RepositoryIdentity> {
    let start = fs::canonicalize(Path::new(cwd)).ok()?;
    let start = if start.is_file() {
        start.parent()?
    } else {
        &start
    };

    for root in start.ancestors() {
        let dot_git = root.join(".git");
        let common_dir = if dot_git.is_dir() {
            fs::canonicalize(&dot_git)
                .ok()
                .filter(|path| looks_like_git_dir(path))
        } else if dot_git.is_file() {
            linked_common_dir(&dot_git)
        } else {
            None
        };
        if let Some(common_dir) = common_dir {
            return identity(root, &common_dir);
        }
        if is_bare_repository(root) {
            return identity(root, root);
        }
    }
    None
}

fn linked_common_dir(dot_git: &Path) -> Option<PathBuf> {
    let contents = fs::read_to_string(dot_git).ok()?;
    if contents.len() > 8192 {
        return None;
    }
    let pointer = contents.trim().strip_prefix("gitdir:")?.trim();
    if pointer.is_empty() || pointer.lines().count() != 1 {
        return None;
    }
    let target = Path::new(pointer);
    let target = if target.is_absolute() {
        target.to_path_buf()
    } else {
        dot_git.parent()?.join(target)
    };
    let target = fs::canonicalize(target).ok()?;

    // A linked worktree's private gitdir is always <common>/worktrees/<name>.
    // Requiring that shape prevents a crafted `.git` file from making us read
    // an arbitrary repository elsewhere on disk.
    let worktrees = target.parent()?;
    if worktrees.file_name()?.to_str()? != "worktrees" {
        return None;
    }
    let expected_common = worktrees.parent()?;
    let common = match fs::read_to_string(target.join("commondir")) {
        Ok(value) => {
            let value = value.trim();
            if value.is_empty() || value.lines().count() != 1 {
                return None;
            }
            fs::canonicalize(target.join(value)).ok()?
        }
        Err(_) => fs::canonicalize(expected_common).ok()?,
    };
    if common != fs::canonicalize(expected_common).ok()? || !looks_like_git_dir(&common) {
        return None;
    }
    Some(common)
}

fn identity(root: &Path, common_dir: &Path) -> Option<RepositoryIdentity> {
    let worktree_root = fs::canonicalize(root).ok()?;
    let common_dir = fs::canonicalize(common_dir).ok()?;
    let remote = origin_url(&common_dir).and_then(|url| normalize_remote(&url));
    let id = remote
        .as_ref()
        .map(|remote| format!("remote:{remote}"))
        .unwrap_or_else(|| format!("git-common-dir:{}", common_dir.to_string_lossy()));
    let name = remote
        .as_deref()
        .and_then(|remote| remote.rsplit('/').next())
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            root.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })?;

    Some(RepositoryIdentity {
        id,
        name,
        worktree_root: worktree_root.to_string_lossy().into_owned(),
    })
}

fn looks_like_git_dir(path: &Path) -> bool {
    path.join("HEAD").is_file() && path.join("config").is_file()
}

fn is_bare_repository(path: &Path) -> bool {
    if !looks_like_git_dir(path) || !path.join("objects").is_dir() {
        return false;
    }
    config_value(&path.join("config"), "core", "bare")
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn origin_url(common_dir: &Path) -> Option<String> {
    config_value(&common_dir.join("config"), "remote \"origin\"", "url")
}

fn config_value(path: &Path, wanted_section: &str, wanted_key: &str) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    let mut section = String::new();
    for raw in contents.lines() {
        let line = raw.trim();
        if let Some(value) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = value.trim().to_ascii_lowercase();
            continue;
        }
        if section != wanted_section.to_ascii_lowercase() || line.starts_with(['#', ';']) {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case(wanted_key) {
            return Some(value.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn normalize_remote(remote: &str) -> Option<String> {
    let value = remote.trim().trim_end_matches('/').trim_end_matches(".git");
    if value.is_empty() {
        return None;
    }

    if let Some((scheme, rest)) = value.split_once("://") {
        if scheme.eq_ignore_ascii_case("file") {
            return None;
        }
        let rest = rest.split(['?', '#']).next()?;
        let (authority, path) = rest.split_once('/')?;
        let host_and_port = authority.rsplit('@').next()?;
        if host_and_port.is_empty() || path.is_empty() {
            return None;
        }
        return Some(format!("{}/{}", host_and_port.to_ascii_lowercase(), path));
    }

    if value
        .find(':')
        .zip(value.find('/'))
        .is_some_and(|(colon, slash)| colon < slash)
    {
        let (authority, path) = value.split_once(':')?;
        let host = authority.rsplit('@').next()?;
        if host.is_empty() || path.is_empty() {
            return None;
        }
        return Some(format!("{}/{}", host.to_ascii_lowercase(), path));
    }

    // Local path remotes are machine-specific and less reliable than the
    // common-directory fallback.
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("mw-repo-{name}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn git_dir(path: &Path, remote: Option<&str>, bare: bool) {
        fs::create_dir_all(path.join("objects")).unwrap();
        fs::write(path.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        let mut config = format!("[core]\n\tbare = {bare}\n");
        if let Some(remote) = remote {
            config.push_str(&format!("[remote \"origin\"]\n\turl = {remote}\n"));
        }
        fs::write(path.join("config"), config).unwrap();
    }

    #[test]
    fn normal_checkout_uses_normalized_origin() {
        let root = temp_dir("normal");
        git_dir(
            &root.join(".git"),
            Some("git@GitHub.com:wuisabel-gif/MemWhale.git"),
            false,
        );
        fs::create_dir(root.join("src")).unwrap();

        let repo = discover(root.join("src").to_str().unwrap()).unwrap();
        assert_eq!(repo.id, "remote:github.com/wuisabel-gif/MemWhale");
        assert_eq!(repo.name, "MemWhale");
        assert_eq!(
            repo.worktree_root,
            fs::canonicalize(&root).unwrap().to_string_lossy()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn linked_worktree_shares_repository_but_keeps_its_root() {
        let root = temp_dir("worktrees");
        let main = root.join("main");
        let linked = root.join("feature");
        let common = main.join(".git");
        let private = common.join("worktrees/feature");
        fs::create_dir_all(&private).unwrap();
        fs::create_dir_all(&linked).unwrap();
        git_dir(
            &common,
            Some("https://user:secret@github.com/org/repo.git"),
            false,
        );
        fs::write(private.join("commondir"), "../..\n").unwrap();
        fs::write(
            linked.join(".git"),
            format!("gitdir: {}\n", private.display()),
        )
        .unwrap();

        let main_repo = discover(main.to_str().unwrap()).unwrap();
        let linked_repo = discover(linked.to_str().unwrap()).unwrap();
        assert_eq!(main_repo.id, linked_repo.id);
        assert_eq!(main_repo.id, "remote:github.com/org/repo");
        assert_ne!(main_repo.worktree_root, linked_repo.worktree_root);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn same_basename_without_remotes_does_not_merge() {
        let root = temp_dir("collision");
        let first = root.join("one/project");
        let second = root.join("two/project");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        git_dir(&first.join(".git"), None, false);
        git_dir(&second.join(".git"), None, false);

        let first = discover(first.to_str().unwrap()).unwrap();
        let second = discover(second.to_str().unwrap()).unwrap();
        assert_eq!(first.name, second.name);
        assert_ne!(first.id, second.id);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_or_untrusted_pointer_is_ignored() {
        let root = temp_dir("malformed");
        let worktree = root.join("worktree");
        let unrelated = root.join("unrelated.git");
        fs::create_dir_all(&worktree).unwrap();
        git_dir(&unrelated, None, false);
        fs::write(
            worktree.join(".git"),
            format!("gitdir: {}\n", unrelated.display()),
        )
        .unwrap();

        assert_eq!(discover(worktree.to_str().unwrap()), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_git_directory_is_ignored() {
        let root = temp_dir("malformed-directory");
        fs::create_dir(root.join(".git")).unwrap();
        assert_eq!(discover(root.to_str().unwrap()), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn remote_ports_remain_part_of_repository_identity() {
        assert_ne!(
            normalize_remote("https://example.com/org/repo.git"),
            normalize_remote("https://example.com:8443/org/repo.git")
        );
    }

    #[test]
    fn bare_repository_is_detected() {
        let root = temp_dir("bare");
        git_dir(&root, None, true);
        let repo = discover(root.to_str().unwrap()).unwrap();
        assert_eq!(repo.name, root.file_name().unwrap().to_string_lossy());
        fs::remove_dir_all(root).unwrap();
    }
}
