//! Conservative, reviewable contradiction flags. This is lexical evidence, not
//! a semantic truth claim; both original records remain untouched.
use crate::Memory;

#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub left_id: i64,
    pub right_id: i64,
    pub score: f32,
    pub reason: String,
}

/// Flag pairs sharing terms but differing on a simple negation cue.
pub fn inspect(left: &Memory, right: &Memory) -> Option<Flag> {
    let a: Vec<String> = left
        .text
        .split_whitespace()
        .map(|x| x.to_lowercase())
        .collect();
    let b: Vec<String> = right
        .text
        .split_whitespace()
        .map(|x| x.to_lowercase())
        .collect();
    let common = a.iter().filter(|x| x.len() > 3 && b.contains(x)).count();
    let neg = |x: &String| {
        matches!(
            x.trim_matches(|c: char| !c.is_alphanumeric()),
            "not" | "never" | "can't" | "cannot" | "avoid"
        )
    };
    if common >= 2 && a.iter().any(neg) != b.iter().any(neg) {
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
}
