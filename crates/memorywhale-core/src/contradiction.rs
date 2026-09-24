//! Conservative, reviewable contradiction flags. This is lexical evidence, not
//! a semantic truth claim; both original records remain untouched.
use crate::engine::fts_tokens;
use crate::Memory;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub left_id: i64,
    pub right_id: i64,
    pub score: f32,
    pub reason: String,
}

/// Flag pairs sharing terms but differing on a simple negation cue.
pub fn inspect(left: &Memory, right: &Memory) -> Option<Flag> {
    // Distinct normalized terms on both sides, so overlap is symmetric and a
    // repeated word cannot satisfy the threshold on its own.
    let terms = |t: &str| -> HashSet<String> { fts_tokens(t).filter(|x| x.len() > 3).collect() };
    let (a, b) = (terms(&left.text), terms(&right.text));
    let common = a.intersection(&b).count();
    let neg = |t: &str| {
        t.split_whitespace().any(|x| {
            matches!(
                x.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric()),
                "not" | "never" | "can't" | "cannot" | "avoid"
            )
        })
    };
    if common >= 2 && neg(&left.text) != neg(&right.text) {
        Some(Flag {
            left_id: left.id,
            right_id: right.id,
            score: common as f32 / a.len().max(b.len()) as f32,
            reason: "lexical overlap with differing negation cue (heuristic; review required)"
                .into(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    fn m(id: i64, text: &str) -> Memory {
        Memory {
            id,
            text: text.into(),
            created_at: Utc::now(),
            last_used: Utc::now(),
            mentions: 0,
            importance: 0.0,
            tags: vec![],
            embedding: None,
            agent: None,
        }
    }
    #[test]
    fn flags_negation_and_overlap() {
        assert!(inspect(
            &m(1, "use cargo build release"),
            &m(2, "do not use cargo build release")
        )
        .is_some());
    }
    #[test]
    fn does_not_claim_every_difference() {
        assert!(inspect(&m(1, "use cargo build"), &m(2, "use cargo test")).is_none());
    }
    #[test]
    fn overlap_is_symmetric_and_deduplicated() {
        let pairs = [
            ("cargo cargo works", "cargo does not work"),
            ("use cargo build release", "do not use cargo build release"),
            ("do not deploy release", "deploy release."),
        ];
        for (x, y) in pairs {
            let (l, r) = (m(1, x), m(2, y));
            let ab = inspect(&l, &r).map(|f| (f.score, f.reason));
            let ba = inspect(&r, &l).map(|f| (f.score, f.reason));
            assert_eq!(ab, ba, "{x:?} vs {y:?}");
        }
        // One shared term repeated is not two shared terms.
        assert!(inspect(&m(1, "cargo cargo works"), &m(2, "cargo does not work")).is_none());
        // Punctuation no longer hides a shared term.
        assert!(inspect(&m(1, "do not deploy release"), &m(2, "deploy release.")).is_some());
    }
}
