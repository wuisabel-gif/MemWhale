//! Rule-of-thumb compaction policy for stored evidence.
//!
//! MemoryWhale keeps three bulky evidence surfaces: session transcripts,
//! command-run output, and the lessons distilled from them. This module is the
//! single rulebook that decides which rows to **Keep** untouched and which to
//! **Compact** (shrink the stored text while preserving the row). It is pure:
//! callers supply facts read from SQLite and receive a tier; all thresholds
//! arrive as arguments so the CLI can expose flags and tests can pin every
//! boundary.
//!
//! The rules of thumb, in evaluation order:
//!
//! 1. **Failures outrank successes.** A failed command or an errored run is
//!    exactly what future-you will search for, so it is never auto-compacted.
//! 2. **A distilled lesson makes bulk redundant.** When an approved lesson was
//!    sourced from a session and enough time has passed, the transcript copy
//!    can shrink — the reasoning lives in the lesson.
//! 3. **Ancient bulk shrinks regardless.** Very old large sessions compact
//!    even without a lesson; `transcript_path` still points at the raw file.
//! 4. **Success noise is compacted first.** Huge successful outputs with no
//!    error fingerprint are the cheapest thing to shrink.
//! 5. **Small things stay.** Below the size thresholds, compaction costs more
//!    than it saves.

use std::fmt;

/// What to do with one row of stored evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tier {
    /// Leave the row exactly as it is.
    Keep,
    /// Shrink the stored text; the reason names which rule fired.
    Compact(Reason),
}

/// Which rule of thumb selected the row for compaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// An approved lesson distilled from this evidence already exists.
    Distilled,
    /// The evidence is old and large; recency no longer protects it.
    StaleLarge,
    /// Large output from a successful command with no error fingerprint.
    SuccessNoise,
}

impl Reason {
    /// Short human-readable label for reports and markers.
    pub fn label(self) -> &'static str {
        match self {
            Reason::Distilled => "distilled into a saved lesson",
            Reason::StaleLarge => "old and larger than the keep threshold",
            Reason::SuccessNoise => "large successful output",
        }
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Facts about one row of `sessions`.
#[derive(Debug, Clone, Copy)]
pub struct SessionFacts {
    /// Stored inline transcript size in bytes (`sessions.byte_count`).
    pub byte_count: i64,
    /// True when the row is an approved (or unreviewed) lesson's source,
    /// i.e. some bookmark row points at this session.
    pub has_distilled_lesson: bool,
    /// Whole days elapsed since `ended_at` (0 for today).
    pub days_since_end: i64,
}

/// Facts about one row of `command_runs`.
#[derive(Debug, Clone, Copy)]
pub struct RunFacts {
    /// The command failed (nonzero exit).
    pub failed: bool,
    /// An error fingerprint was recorded for this run.
    pub has_error_fingerprint: bool,
    /// Combined stdout + stderr bytes.
    pub total_output_bytes: i64,
    /// Some bookmark references this run directly.
    pub referenced_by_bookmark: bool,
}

/// Decide the tier for one session row.
///
/// Rules in order: recording sessions are never touched; small transcripts
/// stay; a distilled lesson after `stale_days`/4 makes the bulk redundant;
/// anything older than `stale_days` and above `min_bytes` shrinks regardless.
pub fn session_tier(facts: SessionFacts, min_bytes: i64, stale_days: i64) -> Tier {
    if facts.byte_count <= min_bytes {
        return Tier::Keep;
    }
    if facts.has_distilled_lesson && facts.days_since_end >= (stale_days / 4).max(1) {
        return Tier::Compact(Reason::Distilled);
    }
    if facts.days_since_end >= stale_days {
        return Tier::Compact(Reason::StaleLarge);
    }
    Tier::Keep
}

/// Decide the tier for one command-run row.
///
/// Failures, fingerprinted errors, bookmarked runs, and anything under
/// `max_output_bytes` are kept. Everything else is success noise.
pub fn run_tier(facts: RunFacts, max_output_bytes: i64) -> Tier {
    if facts.failed || facts.has_error_fingerprint || facts.referenced_by_bookmark {
        return Tier::Keep;
    }
    if facts.total_output_bytes > max_output_bytes {
        return Tier::Compact(Reason::SuccessNoise);
    }
    Tier::Keep
}

/// Build the replacement text for a compacted session transcript.
///
/// The raw file on disk (referenced by `transcript_path`) is untouched; this
/// only shrinks the inline SQLite copy. The marker records the original size
/// so the shrink is auditable.
pub fn compacted_session_transcript(original_bytes: i64) -> String {
    format!(
        "[COMPACTED: {original_bytes} bytes of transcript were here; \
             the raw file is preserved on disk at sessions.transcript_path]"
    )
}

/// Head/tail-truncate one output stream, keeping both ends around a marker.
///
/// Keeps `keep_each` bytes from the head and tail when truncation is needed;
/// inputs already at or under the cap pass through unchanged.
pub fn compact_output_stream(text: &str, max_bytes: i64) -> String {
    let max = max_bytes.max(0) as usize;
    if text.len() <= max {
        return text.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut keep_each = (max / 2).max(1);
    for _ in 0..8 {
        let head = take_prefix(text, keep_each);
        let tail = take_suffix(text, keep_each);
        let omitted = text.len() - head.len() - tail.len();
        let marker = format!("\n[COMPACTED: {omitted} bytes omitted]\n");
        if head.len() + marker.len() + tail.len() <= max {
            return format!("{head}{marker}{tail}");
        }
        keep_each =
            keep_each.saturating_sub(((head.len() + marker.len() + tail.len() - max) / 2).max(1));
    }
    take_prefix("[COMPACTED]", max).to_string()
}

fn take_prefix(text: &str, mut limit: usize) -> &str {
    if limit >= text.len() {
        return text;
    }
    while !text.is_char_boundary(limit) {
        limit -= 1;
    }
    &text[..limit]
}

fn take_suffix(text: &str, start: usize) -> &str {
    if start >= text.len() {
        return "";
    }
    let begin = text.len() - start;
    let mut begin = begin;
    while !text.is_char_boundary(begin) {
        begin += 1;
    }
    &text[begin..]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(bytes: i64, lesson: bool, days: i64) -> SessionFacts {
        SessionFacts {
            byte_count: bytes,
            has_distilled_lesson: lesson,
            days_since_end: days,
        }
    }

    #[test]
    fn small_sessions_stay_regardless_of_age_or_lesson() {
        assert_eq!(
            session_tier(facts(100, true, 365), 256 * 1024, 180),
            Tier::Keep
        );
        assert_eq!(
            session_tier(facts(0, false, 999), 256 * 1024, 180),
            Tier::Keep
        );
    }

    #[test]
    fn recording_is_never_reported_because_size_gate_runs_first() {
        // Recording sessions are filtered by the caller via byte_count/status;
        // the policy itself only sees facts, so document that a fresh recording
        // (tiny bytes) is Keep.
        assert_eq!(
            session_tier(facts(10, false, 0), 256 * 1024, 180),
            Tier::Keep
        );
    }

    #[test]
    fn distilled_lesson_compacts_after_quarter_window() {
        let tier = session_tier(facts(512 * 1024, true, 45), 256 * 1024, 180);
        assert_eq!(tier, Tier::Compact(Reason::Distilled));
    }

    #[test]
    fn distilled_lesson_waits_until_quarter_of_stale_days() {
        // stale_days=180 -> quarter = 45 days; day 44 still keeps.
        assert_eq!(
            session_tier(facts(512 * 1024, true, 44), 256 * 1024, 180),
            Tier::Keep
        );
    }

    #[test]
    fn stale_large_compacts_without_any_lesson() {
        assert_eq!(
            session_tier(facts(512 * 1024, false, 200), 256 * 1024, 180),
            Tier::Compact(Reason::StaleLarge)
        );
    }

    #[test]
    fn large_recent_unlessoned_session_keeps() {
        assert_eq!(
            session_tier(facts(512 * 1024, false, 30), 256 * 1024, 180),
            Tier::Keep
        );
    }

    #[test]
    fn quarter_window_floors_at_one_day() {
        // stale_days=3 -> quarter=0 -> floor at 1: day 1 with a lesson compacts.
        assert_eq!(
            session_tier(facts(512 * 1024, true, 1), 256 * 1024, 3),
            Tier::Compact(Reason::Distilled)
        );
    }

    fn run(failed: bool, fp: bool, bytes: i64, referenced: bool) -> RunFacts {
        RunFacts {
            failed,
            has_error_fingerprint: fp,
            total_output_bytes: bytes,
            referenced_by_bookmark: referenced,
        }
    }

    #[test]
    fn failures_and_fingerprints_always_keep() {
        assert_eq!(
            run_tier(run(true, false, 10_000_000, false), 64 * 1024),
            Tier::Keep
        );
        assert_eq!(
            run_tier(run(false, true, 10_000_000, false), 64 * 1024),
            Tier::Keep
        );
    }

    #[test]
    fn bookmarked_runs_keep_even_when_huge_successes() {
        assert_eq!(
            run_tier(run(false, false, 10_000_000, true), 64 * 1024),
            Tier::Keep
        );
    }

    #[test]
    fn big_successful_output_compacts() {
        assert_eq!(
            run_tier(run(false, false, 128 * 1024, false), 64 * 1024),
            Tier::Compact(Reason::SuccessNoise)
        );
    }

    #[test]
    fn small_successful_output_keeps() {
        assert_eq!(
            run_tier(run(false, false, 1024, false), 64 * 1024),
            Tier::Keep
        );
        assert_eq!(
            run_tier(run(false, false, 64 * 1024, false), 64 * 1024),
            Tier::Keep
        );
    }

    #[test]
    fn compacted_transcript_marker_records_original_size() {
        let marker = compacted_session_transcript(912_345);
        assert!(marker.contains("912345"));
        assert!(marker.contains("[COMPACTED:"));
        assert!(marker.contains("preserved on disk"));
    }

    #[test]
    fn output_stream_passthrough_under_cap() {
        assert_eq!(compact_output_stream("hello", 100), "hello");
        assert_eq!(compact_output_stream("", 100), "");
    }

    #[test]
    fn output_stream_head_tail_with_marker_over_cap() {
        let big = format!("HEAD{}\nTAIL", "x".repeat(10_000));
        let out = compact_output_stream(&big, 2048);
        assert!(out.starts_with("HEAD"));
        assert!(out.ends_with("\nTAIL"));
        assert!(out.contains("[COMPACTED: "));
        assert!(out.len() < big.len());
    }

    #[test]
    fn output_stream_truncation_respects_utf8_boundaries() {
        let emoji = "🦈".repeat(5_000); // 4 bytes each, 20_000 total
        let out = compact_output_stream(&emoji, 1024);
        // Must remain valid UTF-8 (String type guarantees it, but the slicing
        // inside must not panic).
        assert!(out.contains("[COMPACTED: "));
        assert!(out.ends_with("🦈") || out.ends_with('\n') || out.ends_with('🦈'));
    }

    #[test]
    fn zero_max_returns_empty_output() {
        let out = compact_output_stream("anything", 0);
        assert!(out.is_empty());
    }

    #[test]
    fn compacted_output_is_bounded_and_idempotent() {
        let input = "x".repeat(10_000);
        let once = compact_output_stream(&input, 512);
        let twice = compact_output_stream(&once, 512);
        assert!(once.len() <= 512);
        assert_eq!(once, twice);
    }
}
