//! Fuzzy matching for the command palette: every word of the query has to appear in order,
//! not necessarily adjacent, in the candidate.

const MATCH: u32 = 16;
const WORD_START: u32 = 32;
const CONSECUTIVE: u32 = 24;
const GAP: u32 = 1;
const MAX_GAP_PENALTY: u32 = 8;

pub fn score(query: &str, candidate: &str) -> Option<u32> {
    let haystack: Vec<char> = candidate.chars().collect();
    query.split_whitespace().try_fold(0u32, |total, word| {
        Some(total + word_score(word, &haystack)?)
    })
}

/// Indices of the matching items, best first; equal scores keep the input order.
pub fn rank<T>(query: &str, items: &[T], text: impl Fn(&T) -> &str) -> Vec<usize> {
    let mut scored: Vec<(usize, u32)> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| score(query, text(item)).map(|score| (index, score)))
        .collect();
    scored.sort_by_key(|&(_, score)| std::cmp::Reverse(score));
    scored.into_iter().map(|(index, _)| index).collect()
}

fn word_score(word: &str, haystack: &[char]) -> Option<u32> {
    let needle: Vec<char> = word.chars().collect();
    let case_sensitive = needle.iter().any(|c| c.is_uppercase());
    let eq = |a: char, b: char| {
        if case_sensitive {
            a == b
        } else {
            a.to_lowercase().eq(b.to_lowercase())
        }
    };
    (0..haystack.len())
        .filter(|&start| eq(needle[0], haystack[start]))
        .filter_map(|start| score_from(&needle, haystack, start, &eq))
        .max()
}

/// Greedy from `start`; greedy leftmost matching finds a subsequence whenever one exists there.
fn score_from(
    needle: &[char],
    haystack: &[char],
    start: usize,
    eq: &dyn Fn(char, char) -> bool,
) -> Option<u32> {
    let mut score = 0u32;
    let mut next = start;
    let mut previous: Option<usize> = None;
    for &c in needle {
        let found = (next..haystack.len()).find(|&index| eq(c, haystack[index]))?;
        score += MATCH;
        if is_word_start(haystack, found) {
            score += WORD_START;
        }
        match previous {
            Some(p) if found == p + 1 => score += CONSECUTIVE,
            Some(p) => {
                let gap = u32::try_from(found - p - 1).unwrap_or(u32::MAX);
                score = score.saturating_sub(GAP.saturating_mul(gap).min(MAX_GAP_PENALTY));
            }
            None => {}
        }
        previous = Some(found);
        next = found + 1;
    }
    Some(score)
}

fn is_word_start(haystack: &[char], index: usize) -> bool {
    index == 0 || matches!(haystack[index - 1], ' ' | '-' | '_' | ':' | '/' | '.')
}

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn every_word_must_match_in_order() {
        assert!(score("mig up", "run migrate-up").is_some());
        assert!(
            score("up mig", "run migrate-up").is_some(),
            "words match independently"
        );
        assert!(score("pu", "run migrate-up").is_none());
        assert!(score("mig x", "run migrate-up").is_none());
    }

    #[test]
    fn word_starts_and_runs_rank_above_scattered_matches() {
        let items = ["stop db", "brackets", "restart backend"];
        assert_eq!(rank("back", &items, |item| item), [2, 1]);
    }

    #[test]
    fn upper_case_makes_a_word_case_sensitive() {
        assert!(score("db", "stop DB").is_some());
        assert!(score("DB", "stop db").is_none());
        assert!(score("DB", "stop DB").is_some());
    }

    #[test]
    fn an_empty_query_keeps_every_item_in_order() {
        assert_eq!(rank("", &["b", "a"], |item| item), [0, 1]);
        assert_eq!(rank("   ", &["b", "a"], |item| item), [0, 1]);
    }

    #[test]
    fn ties_keep_the_input_order() {
        assert_eq!(rank("x", &["ax", "bx"], |item| item), [0, 1]);
    }

    #[quickcheck]
    fn every_subsequence_of_a_candidate_matches(candidate: String, picks: Vec<bool>) -> TestResult {
        let candidate = candidate.to_lowercase();
        let needle: String = candidate
            .chars()
            .zip(picks.iter().copied().chain(std::iter::repeat(false)))
            .filter(|(c, pick)| *pick && !c.is_whitespace())
            .map(|(c, _)| c)
            .collect();
        if needle.is_empty() {
            return TestResult::discard();
        }
        TestResult::from_bool(score(&needle, &candidate).is_some())
    }

    #[quickcheck]
    fn never_panics(query: String, candidate: String) -> bool {
        let _ = score(&query, &candidate);
        true
    }
}
