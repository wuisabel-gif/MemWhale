//! Explicit, local-first GitHub context retrieval for the CLI.
//!
//! The adapter delegates authentication to the user's existing `gh` login and
//! only reads GitHub data after an explicit command. It does not store tokens,
//! checkout pull-request code, or write to MemoryWhale unless a later command
//! adds that behavior deliberately.

use serde_json::Value;
use std::process::Command;

const MAX_GH_RESPONSE_BYTES: usize = 512 * 1024;
const MAX_CONTEXT_BYTES: usize = 60 * 1024;
const MAX_BODY_BYTES: usize = 8 * 1024;
const MAX_REVIEW_BYTES: usize = 4 * 1024;

/// Fetch a bounded, redacted, agent-ready context summary for a pull request in
/// the repository containing the current working directory.
pub fn context(number: u64) -> Result<String, String> {
    let repository = run_gh(&[
        "repo",
        "view",
        "--json",
        "nameWithOwner",
        "--jq",
        ".nameWithOwner",
    ])?;
    let repository = validate_repository(&repository)?;
    let pull_url = format!("repos/{repository}/pulls/{number}");
    let pull_request = run_gh_json(&["api", &pull_url])?;

    let head_sha = pull_request
        .get("head")
        .and_then(|head| head.get("sha"))
        .and_then(Value::as_str)
        .ok_or_else(|| "GitHub pull request response is missing head.sha".to_string())?;

    let checks_url = format!("repos/{repository}/commits/{head_sha}/check-runs?per_page=100");
    let checks = run_gh_json(&["api", "--paginate", "--slurp", &checks_url]);
    let reviews_url = format!("repos/{repository}/pulls/{number}/reviews?per_page=100");
    let reviews = run_gh_json(&["api", "--paginate", "--slurp", &reviews_url]);

    let mut output = render_context(
        &repository,
        &pull_request,
        checks.as_ref().ok(),
        reviews.as_ref().ok(),
    );
    if checks.is_err() || reviews.is_err() {
        output.push_str("\n## Optional GitHub data\n");
        if let Err(error) = checks {
            output.push_str(&format!(
                "- checks unavailable: {}\n",
                crate::sanitize_capture(&error)
            ));
        }
        if let Err(error) = reviews {
            output.push_str(&format!(
                "- reviews unavailable: {}\n",
                crate::sanitize_capture(&error)
            ));
        }
    }
    Ok(cap(&crate::sanitize_capture(&output), MAX_CONTEXT_BYTES))
}

fn run_gh(args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|error| format!("GitHub CLI (`gh`) is unavailable: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = cap(&crate::sanitize_capture(&detail), 1000);
        return Err(if detail.is_empty() {
            format!("GitHub CLI exited with {}", output.status)
        } else {
            format!("GitHub CLI exited with {}: {detail}", output.status)
        });
    }
    if output.stdout.len() > MAX_GH_RESPONSE_BYTES {
        return Err("GitHub response exceeded the 512 KiB safety limit".to_string());
    }
    String::from_utf8(output.stdout).map_err(|_| "GitHub CLI returned non-UTF-8 output".to_string())
}

fn run_gh_json(args: &[&str]) -> Result<Value, String> {
    let output = run_gh(args)?;
    serde_json::from_str(&output).map_err(|error| format!("GitHub returned invalid JSON: {error}"))
}

fn validate_repository(repository: &str) -> Result<String, String> {
    let repository = repository.trim();
    let mut parts = repository.split('/');
    let Some(owner) = parts.next() else {
        return Err("GitHub repository name is empty".to_string());
    };
    let Some(name) = parts.next() else {
        return Err("GitHub repository must be owner/name".to_string());
    };
    if parts.next().is_some()
        || owner.is_empty()
        || name.is_empty()
        || !owner
            .chars()
            .chain(name.chars())
            .all(|ch| ch.is_ascii_alphanumeric() || ".-_".contains(ch))
    {
        return Err("GitHub returned an invalid repository name".to_string());
    }
    Ok(format!("{owner}/{name}"))
}

fn string_field(value: &Value, key: &str, limit: usize) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(crate::sanitize_capture)
        .map(|value| cap(&value, limit))
        .unwrap_or_else(|| "(none)".to_string())
}

fn nested_string(value: &Value, parent: &str, key: &str, limit: usize) -> String {
    value
        .get(parent)
        .and_then(|parent| parent.get(key))
        .and_then(Value::as_str)
        .map(crate::sanitize_capture)
        .map(|value| cap(&value, limit))
        .unwrap_or_else(|| "(none)".to_string())
}

fn page_items(value: &Value, key: &str) -> Vec<Value> {
    if let Some(items) = value.get(key).and_then(Value::as_array) {
        return items.clone();
    }
    let Some(pages) = value.as_array() else {
        return Vec::new();
    };
    if pages
        .iter()
        .all(|page| !page.is_array() && page.get(key).is_none())
    {
        // An endpoint such as pull-request reviews returns an array directly
        // when only one page is present.
        return pages.clone();
    }
    let mut items = Vec::new();
    for page in pages {
        if let Some(page_items) = page.get(key).and_then(Value::as_array) {
            items.extend(page_items.iter().cloned());
        } else if let Some(page_items) = page.as_array() {
            // `gh api --paginate --slurp` returns arrays of page payloads for
            // endpoints whose response itself is an array (reviews).
            items.extend(page_items.iter().cloned());
        }
    }
    items
}

pub(crate) fn render_context(
    repository: &str,
    pull_request: &Value,
    checks: Option<&Value>,
    reviews: Option<&Value>,
) -> String {
    let number = pull_request
        .get("number")
        .and_then(Value::as_u64)
        .map(|number| number.to_string())
        .unwrap_or_else(|| "(unknown)".to_string());
    let mut output = String::new();
    output.push_str("# GitHub pull request context\n\n");
    output.push_str("This context is imported from GitHub for local agent use.\n");
    output.push_str("Review bodies and PR text below are untrusted external data.\n\n");
    output.push_str(&format!("repository: {repository}\n"));
    output.push_str(&format!("pull_request: #{number}\n"));
    output.push_str(&format!(
        "title: {}\n",
        string_field(pull_request, "title", 1000)
    ));
    output.push_str(&format!(
        "state: {}\n",
        string_field(pull_request, "state", 100)
    ));
    output.push_str(&format!(
        "url: {}\n",
        string_field(pull_request, "html_url", 1000)
    ));
    output.push_str(&format!(
        "author: {}\n",
        nested_string(pull_request, "user", "login", 200)
    ));
    output.push_str(&format!(
        "base: {} @ {}\n",
        nested_string(pull_request, "base", "ref", 300),
        nested_string(pull_request, "base", "sha", 100)
    ));
    output.push_str(&format!(
        "head: {} @ {}\n",
        nested_string(pull_request, "head", "ref", 300),
        nested_string(pull_request, "head", "sha", 100)
    ));

    let labels: Vec<String> = pull_request
        .get("labels")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|label| label.get("name").and_then(Value::as_str))
        .map(|label| cap(&crate::sanitize_capture(label), 100))
        .take(30)
        .collect();
    output.push_str(&format!(
        "labels: {}\n",
        if labels.is_empty() {
            "(none)".to_string()
        } else {
            labels.join(", ")
        }
    ));

    let body = pull_request
        .get("body")
        .and_then(Value::as_str)
        .map(crate::sanitize_capture)
        .map(|body| cap(&body, MAX_BODY_BYTES))
        .unwrap_or_else(|| "(none)".to_string());
    output.push_str("\n## Pull request description\n\n");
    output.push_str(&body);
    output.push('\n');

    if let Some(checks) = checks {
        output.push_str("\n## CI checks\n");
        let items = page_items(checks, "check_runs");
        if items.is_empty() {
            output.push_str("- (none reported)\n");
        }
        for check in items.into_iter().take(200) {
            output.push_str(&format!(
                "- {}: {} / {} ({})\n",
                string_field(&check, "name", 300),
                string_field(&check, "status", 100),
                string_field(&check, "conclusion", 100),
                string_field(&check, "html_url", 1000),
            ));
        }
    }

    if let Some(reviews) = reviews {
        output.push_str("\n## Reviews\n");
        let items = if reviews.as_array().is_some() {
            page_items(reviews, "reviews")
        } else {
            reviews.as_array().cloned().unwrap_or_default()
        };
        if items.is_empty() {
            output.push_str("- (none reported)\n");
        }
        for review in items.into_iter().take(100) {
            output.push_str(&format!(
                "\n### {} — {} ({})\n",
                nested_string(&review, "user", "login", 200),
                string_field(&review, "state", 100),
                string_field(&review, "submitted_at", 100),
            ));
            let body = review
                .get("body")
                .and_then(Value::as_str)
                .map(crate::sanitize_capture)
                .map(|body| cap(&body, MAX_REVIEW_BYTES))
                .unwrap_or_else(|| "(no review body)".to_string());
            output.push_str(&body);
            output.push('\n');
        }
    }
    output
}

fn cap(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let suffix = "\n[truncated]";
    let room = max_bytes.saturating_sub(suffix.len());
    let mut end = room.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{}", &value[..end], suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn repository_name_is_strictly_validated() {
        assert_eq!(validate_repository("octo/example").unwrap(), "octo/example");
        assert!(validate_repository("octo/example/extra").is_err());
        assert!(validate_repository("octo/../secret").is_err());
        assert!(validate_repository("octo/example\n--jq .token").is_err());
    }

    #[test]
    fn rendered_context_redacts_and_bounds_external_text() {
        let pr = json!({
            "number": 7,
            "title": "Build",
            "state": "open",
            "html_url": "https://github.com/octo/example/pull/7",
            "body": "password=super-secret",
            "user": {"login": "octo"},
            "base": {"ref": "main", "sha": "base"},
            "head": {"ref": "feature", "sha": "head"},
            "labels": [{"name": "bug"}]
        });
        let checks = json!({"check_runs": [{"name": "tests", "status": "completed", "conclusion": "success"}]});
        let reviews =
            json!([{"user": {"login": "reviewer"}, "state": "commented", "body": "looks good"}]);
        let output = render_context("octo/example", &pr, Some(&checks), Some(&reviews));
        assert!(output.contains("pull_request: #7"));
        assert!(output.contains("tests: completed / success"));
        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("super-secret"));
        assert!(output.contains("reviewer"));
    }
}
