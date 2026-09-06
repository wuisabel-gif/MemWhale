use super::*;
use serde_json::json;

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn pull_request() -> Value {
    json!({
        "number": 7,
        "title": "Build",
        "state": "open",
        "html_url": "https://github.com/octo/example/pull/7",
        "body": "PR description",
        "user": {"login": "octo"},
        "base": {"ref": "main", "sha": SHA},
        "head": {"ref": "feature", "sha": SHA},
        "labels": [{"name": "bug"}]
    })
}

fn check(index: usize) -> Value {
    json!({"name": format!("check-{index}"), "status": "completed", "conclusion": "success"})
}

fn status(index: usize) -> Value {
    json!({"context": format!("status-{index}"), "state": "success"})
}

fn review() -> Value {
    json!({"user": {"login": "reviewer"}, "state": "COMMENTED", "body": "review survives"})
}

#[test]
fn repository_and_head_are_validated_before_path_construction() {
    assert_eq!(
        validate_repository(" octo/example\n").unwrap(),
        "octo/example"
    );
    for invalid in [
        "octo/example/extra",
        "octo/../secret",
        "octo/example\n--jq .token",
        "../example",
        "octo/..",
        "./example",
        "octo/.",
        "octo/example?x=y",
        "octo/%2e%2e",
        "",
    ] {
        assert!(validate_repository(invalid).is_err(), "{invalid:?}");
    }
    assert!(validate_repository(&format!("octo/{}", "x".repeat(300))).is_err());
    assert_eq!(validate_pull_request(&pull_request(), 7).unwrap(), SHA);
    for sha in [
        "",
        "head",
        "../reviews?x=y",
        "--help",
        &"g".repeat(40),
        &"a".repeat(41),
    ] {
        let mut pr = pull_request();
        pr["head"]["sha"] = json!(sha);
        assert!(validate_pull_request(&pr, 7).is_err());
    }
    assert!(validate_pull_request(&pull_request(), 8).is_err());
    assert!(validate_pull_request(&json!([]), 7).is_err());
}

#[test]
fn malformed_optional_responses_are_not_empty_successes() {
    for kind in [PageKind::Checks, PageKind::Statuses, PageKind::Reviews] {
        for invalid in [Value::Null, json!({"message": "forbidden"}), json!([17])] {
            assert!(kind.items(&invalid).is_err());
        }
    }
    for invalid in [
        json!({"total_count": "1", "check_runs": [check(0)]}),
        json!({"total_count": 0, "check_runs": [check(0)]}),
        json!({"total_count": 1, "check_runs": [{"name": "x", "status": 17}]}),
        json!({"total_count": 1, "check_runs": {}}),
        json!({"total_count": 101, "check_runs": (0..101).map(check).collect::<Vec<_>>()}),
    ] {
        assert!(PageKind::Checks.items(&invalid).is_err());
    }
    assert!(PageKind::Statuses
        .items(&json!({"total_count": 1, "statuses": [{}]}))
        .is_err());
    assert!(PageKind::Reviews.items(&json!([{"state": null}])).is_err());
}

#[test]
fn sections_reserve_independent_failure_and_truncation_markers() {
    let checks = Section {
        items: vec![
            json!({"name": "x".repeat(300), "status": "completed", "html_url": "x".repeat(1000)});
            200
        ],
        unavailable: Some("check page failed".into()),
        truncated: true,
    };
    let statuses = Section {
        items: vec![
            json!({"context": "x".repeat(300), "state": "pending", "description": "x".repeat(1000)});
            200
        ],
        unavailable: Some("status page failed".into()),
        truncated: true,
    };
    let reviews = Section {
        items: vec![
            json!({"user": {"login": "reviewer"}, "state": "COMMENTED", "body": "x".repeat(5000)});
            100
        ],
        unavailable: Some("review page failed".into()),
        truncated: true,
    };
    let mut pr = pull_request();
    for field in ["title", "state", "body", "html_url"] {
        pr[field] = json!("x".repeat(20_000));
    }
    pr["labels"] = json!(vec![json!({"name": "x".repeat(200)}); 30]);
    for (output, limit, error) in [
        (
            checks_section(&checks),
            MAX_CHECKS_SECTION_BYTES,
            "check page failed",
        ),
        (
            statuses_section(&statuses),
            MAX_STATUSES_SECTION_BYTES,
            "status page failed",
        ),
        (
            reviews_section(&reviews),
            MAX_REVIEWS_SECTION_BYTES,
            "review page failed",
        ),
    ] {
        assert!(output.len() <= limit);
        assert!(output.contains("section truncated"));
        assert!(output.contains(error));
    }
    let output = render_context("octo/example", &pr, &checks, &statuses, &reviews);
    assert!(output.len() <= MAX_CONTEXT_BYTES);
    assert_eq!(output.matches("section truncated").count(), 3);
    assert!(output.contains("review page failed"));
    assert!(output.contains("reviewer"));
}

#[test]
fn cap_obeys_small_limits_and_utf8_boundaries() {
    for limit in 0..30 {
        let output = cap(&"🦀".repeat(20), limit);
        assert!(output.len() <= limit);
    }
}

#[cfg(unix)]
mod fake_gh {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    // An injected executable, not PATH/env mutation: tests can run in parallel
    // with each other and with the rest of the CLI suite, without network/login.
    struct FakeGh {
        directory: PathBuf,
    }

    impl FakeGh {
        fn new(script: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let directory = std::env::temp_dir().join(format!(
                "memorywhale-github-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&directory).unwrap();
            let fake = Self { directory };
            fake.put("gh", &format!(
                "#!/bin/sh\nset -eu\ncd \"${{0%/*}}\"\nprintf '%s\\n' \"$$\" > pid\nprintf '%s\\n' \"$*\" >> calls\n{script}\n"
            ));
            fs::set_permissions(fake.directory.join("gh"), fs::Permissions::from_mode(0o700))
                .unwrap();
            fake
        }

        fn fixture() -> Self {
            let fake = Self::new(
                r#"
reply() {
    if [ -f "$1.error" ]; then
        cat "$1.error" >&2
        exit 23
    fi
    cat "$1"
}
case "$*" in
    'repo view --json nameWithOwner --jq .nameWithOwner') reply repository ;;
    'api repos/octo/example/pulls/7') reply pull ;;
    'api repos/octo/example/commits/'*'/check-runs?per_page=100&page='*) reply "checks-${2##*page=}" ;;
    'api repos/octo/example/commits/'*'/status?per_page=100&page='*) reply "statuses-${2##*page=}" ;;
    'api repos/octo/example/pulls/7/reviews?per_page=100&page='*) reply "reviews-${2##*page=}" ;;
    *) printf 'unexpected fake gh arguments: %s\n' "$*" >&2; exit 91 ;;
esac
"#,
            );
            fake.put("repository", "octo/example\n");
            fake.json("pull", &pull_request());
            fake.json(
                "checks-1",
                &json!({"total_count": 1, "check_runs": [check(0)]}),
            );
            fake.json(
                "statuses-1",
                &json!({"total_count": 1, "statuses": [{
                    "context": "CodeRabbit", "state": "success", "description": "Review completed",
                    "target_url": "https://example.test/review"
                }]}),
            );
            fake.json("reviews-1", &json!([review()]));
            fake
        }

        fn put(&self, name: &str, content: &str) {
            fs::write(self.directory.join(name), content).unwrap();
        }

        fn json(&self, name: &str, value: &Value) {
            self.put(name, &value.to_string());
        }

        fn gh(&self) -> Gh {
            Gh {
                program: self.directory.join("gh"),
                runtime: Duration::from_secs(3),
            }
        }

        fn calls(&self) -> Vec<String> {
            fs::read_to_string(self.directory.join("calls"))
                .unwrap_or_default()
                .lines()
                .map(str::to_string)
                .collect()
        }

        fn assert_reaped(&self) {
            let pid: libc::pid_t = fs::read_to_string(self.directory.join("pid"))
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            let mut status = 0;
            // SAFETY: status is writable; WNOHANG never waits on a live child.
            assert_eq!(
                unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) },
                -1
            );
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ECHILD)
            );
            assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "fake gh is still alive");
        }
    }

    impl Drop for FakeGh {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }

    #[test]
    fn fetches_check_runs_and_classic_status_contexts_independently() {
        let fake = FakeGh::fixture();
        let output = context_with(7, &fake.gh()).unwrap();
        assert!(output.contains("pull_request: #7"));
        assert!(output.contains("check-0: completed / success"));
        assert!(output.contains("CodeRabbit: success — Review completed"));
        assert!(output.contains("review survives"));
        assert!(!output.contains("unavailable"));
        assert!(!output.contains("section truncated"));
        let calls = fake.calls();
        assert_eq!(calls.len(), 5);
        assert!(calls
            .iter()
            .any(|call| call.contains(&format!("commits/{SHA}/status?"))));
        assert!(calls
            .iter()
            .all(|call| !call.contains("--paginate") && !call.contains("--slurp")));
        fake.assert_reaped();
    }

    #[test]
    fn paginates_checks_and_statuses_with_explicit_item_limits() {
        let fake = FakeGh::fixture();
        for page in 1..=2 {
            let indices = (page - 1) * 100..page * 100;
            fake.json(&format!("checks-{page}"), &json!({"total_count": 201, "check_runs": indices.clone().map(check).collect::<Vec<_>>()}));
            fake.json(
                &format!("statuses-{page}"),
                &json!({"total_count": 201, "statuses": indices.map(status).collect::<Vec<_>>()}),
            );
        }
        let output = context_with(7, &fake.gh()).unwrap();
        assert!(output.contains("check-100:"));
        assert!(output.contains("status-100:"));
        assert_eq!(output.matches("section truncated").count(), 2);
        assert!(output.contains("review survives"));
        assert_eq!(fake.calls().len(), 7);
        assert!(fake.calls().iter().all(|call| !call.contains("page=3")));
    }

    #[test]
    fn a_known_complete_full_page_is_not_marked_truncated() {
        let fake = FakeGh::fixture();
        fake.json(
            "checks-1",
            &json!({"total_count": 100, "check_runs": (0..100).map(check).collect::<Vec<_>>()}),
        );
        let output = context_with(7, &fake.gh()).unwrap();
        assert!(!output.contains("section truncated"));
        assert_eq!(fake.calls().len(), 5);
    }

    #[test]
    fn a_full_review_page_marks_the_unknown_remainder() {
        let fake = FakeGh::fixture();
        fake.json("reviews-1", &json!(vec![review(); 100]));
        let output = context_with(7, &fake.gh()).unwrap();
        assert!(output
            .split("## Reviews")
            .nth(1)
            .unwrap()
            .contains("section truncated"));
        assert_eq!(fake.calls().len(), 5);
    }

    #[test]
    fn later_page_errors_keep_partial_results_and_other_sources() {
        let fake = FakeGh::fixture();
        fake.json(
            "checks-1",
            &json!({"total_count": 101, "check_runs": (0..100).map(check).collect::<Vec<_>>()}),
        );
        fake.put("checks-2.error", "checks access denied");
        let output = context_with(7, &fake.gh()).unwrap();
        let checks = output
            .split("## CI checks")
            .nth(1)
            .unwrap()
            .split("## Commit statuses")
            .next()
            .unwrap();
        assert!(checks.contains("check-0:"));
        assert!(checks.contains("section truncated"));
        assert!(checks.contains("unavailable:"));
        assert!(checks.contains("checks access denied"));
        assert!(output.contains("CodeRabbit: success"));
        assert!(output.contains("review survives"));
    }

    #[test]
    fn malformed_and_failed_sources_are_not_reported_as_empty() {
        let fake = FakeGh::fixture();
        fake.json("checks-1", &json!({"message": "not a checks page"}));
        fake.put("statuses-1", "invalid json");
        fake.put("reviews-1.error", "review permission denied");
        let output = context_with(7, &fake.gh()).unwrap();
        assert_eq!(output.matches("unavailable:").count(), 3);
        assert!(!output.contains("none reported"));
        assert!(output.contains("invalid paginated response shape"));
        assert!(output.contains("invalid JSON"));
        assert!(output.contains("review permission denied"));
        assert!(output.contains("PR description"));
    }

    #[test]
    fn empty_sources_are_available_without_truncation() {
        let fake = FakeGh::fixture();
        fake.json("checks-1", &json!({"total_count": 0, "check_runs": []}));
        fake.json("statuses-1", &json!({"total_count": 0, "statuses": []}));
        fake.json("reviews-1", &json!([]));
        let output = context_with(7, &fake.gh()).unwrap();
        assert_eq!(output.matches("none reported").count(), 3);
        assert!(!output.contains("unavailable"));
        assert!(!output.contains("truncated"));
    }

    #[test]
    fn response_byte_budget_is_shared_across_pages_not_sources() {
        let fake = FakeGh::fixture();
        for page in 1..=2 {
            fake.json(
                &format!("checks-{page}"),
                &json!({
                    "total_count": 200, "ignored": "x".repeat(270 * 1024),
                    "check_runs": ((page - 1) * 100..page * 100).map(check).collect::<Vec<_>>()
                }),
            );
        }
        let output = context_with(7, &fake.gh()).unwrap();
        assert!(output.contains("check-0:"));
        assert!(!output.contains("check-100:"));
        assert!(output.contains("stdout safety limit"));
        assert!(output.contains("section truncated"));
        assert!(output.contains("CodeRabbit: success"));
        assert!(output.contains("review survives"));
        assert_eq!(fake.calls().len(), 6);
        assert!(output.len() <= MAX_CONTEXT_BYTES);
    }

    #[test]
    fn untrusted_text_is_redacted_and_terminal_controls_are_neutralized() {
        let fake = FakeGh::fixture();
        let hostile = "\u{1b}]52;c;clipboard\u{7}\rpassword=super-secret\n";
        let mut pr = pull_request();
        pr["body"] = json!(hostile);
        pr["title"] = json!(hostile);
        fake.json("pull", &pr);
        fake.json("statuses-1", &json!({"total_count": 1, "statuses": [{
            "context": "CodeRabbit", "state": "success", "description": hostile, "target_url": hostile
        }]}));
        fake.json(
            "reviews-1",
            &json!([{"user": {"login": hostile}, "state": "COMMENTED", "body": hostile}]),
        );
        fake.put("checks-1.error", hostile);
        let output = context_with(7, &fake.gh()).unwrap();
        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("super-secret"));
        assert!(output
            .chars()
            .all(|ch| !ch.is_control() || matches!(ch, '\n' | '\t')));
        assert!(output.contains("CodeRabbit: success"));
        assert!(output.contains("unavailable:"));
    }

    #[test]
    fn invalid_pr_metadata_stops_before_any_commit_or_review_requests() {
        for invalid in [
            json!([]),
            json!({"number": 8, "head": {"sha": SHA}}),
            json!({"number": 7, "head": {"sha": "../../pulls/8?x=y"}}),
        ] {
            let fake = FakeGh::fixture();
            fake.json("pull", &invalid);
            assert!(context_with(7, &fake.gh()).is_err());
            assert_eq!(fake.calls().len(), 2);
        }
        let fake = FakeGh::fixture();
        assert!(context_with(0, &fake.gh()).is_err());
        assert!(fake.calls().is_empty());
        fake.put("repository", "../example");
        assert!(context_with(7, &fake.gh()).is_err());
        assert_eq!(fake.calls().len(), 1);
    }

    #[test]
    fn stdout_and_stderr_caps_kill_and_reap_even_with_inherited_pipes() {
        for stream in ["stdout", "stderr"] {
            let redirect = if stream == "stderr" { " >&2" } else { "" };
            let fake = FakeGh::new(&format!(
                "sleep 30 &\n(dd if=/dev/zero bs=8192 count=65 2>/dev/null){redirect}\nwait"
            ));
            let started = Instant::now();
            let error = fake.gh().run(&[], MAX_GH_RESPONSE_BYTES).unwrap_err();
            assert!(error.contains(&format!("{stream} safety limit")), "{error}");
            assert!(started.elapsed() < Duration::from_secs(3));
            fake.assert_reaped();
        }
    }

    #[test]
    fn closed_pipes_do_not_remove_the_process_deadline() {
        let fake = FakeGh::new("exec 1>&- 2>&-\nexec sleep 30");
        // Use the normal fixture budget: a one-second startup deadline races
        // process scheduling when the whole workspace suite runs in parallel.
        let gh = fake.gh();
        let started = Instant::now();
        assert!(gh
            .run(&[], MAX_GH_RESPONSE_BYTES)
            .unwrap_err()
            .contains("runtime limit"));
        assert!(started.elapsed() < Duration::from_secs(10));
        fake.assert_reaped();
    }

    #[test]
    fn exited_child_with_descendant_holding_pipes_cannot_hang_cleanup() {
        let fake = FakeGh::new("sleep 30 &\nexit 0");
        let gh = fake.gh();
        let started = Instant::now();
        assert!(gh
            .run(&[], MAX_GH_RESPONSE_BYTES)
            .unwrap_err()
            .contains("runtime limit"));
        assert!(started.elapsed() < Duration::from_secs(10));
        fake.assert_reaped();
    }

    #[test]
    fn exact_output_limit_is_allowed_but_the_next_byte_is_rejected() {
        let fake = FakeGh::new("printf abcd");
        assert_eq!(fake.gh().run(&[], 4).unwrap(), "abcd");
        assert!(fake
            .gh()
            .run(&[], 3)
            .unwrap_err()
            .contains("stdout safety limit"));
        fake.assert_reaped();
    }

    #[test]
    fn subprocess_errors_and_non_utf8_are_reported_and_reaped() {
        let fake = FakeGh::new("printf '\\033[2Jpassword=super-secret\\007' >&2\nexit 23");
        let error = fake.gh().run(&[], MAX_GH_RESPONSE_BYTES).unwrap_err();
        assert!(error.contains("23"));
        assert!(error.contains("[REDACTED]"));
        assert!(!error.contains("super-secret"));
        assert!(!error.chars().any(char::is_control));
        fake.assert_reaped();
        let fake = FakeGh::new("printf '\\377'");
        assert!(fake
            .gh()
            .run(&[], MAX_GH_RESPONSE_BYTES)
            .unwrap_err()
            .contains("non-UTF-8"));
        fake.assert_reaped();
        let gh = Gh {
            program: fake.directory.join("missing-gh"),
            runtime: Duration::from_secs(1),
        };
        assert!(gh
            .run(&[], MAX_GH_RESPONSE_BYTES)
            .unwrap_err()
            .contains("unavailable"));
    }
}
