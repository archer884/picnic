extern crate regex;

use regex::Regex;
use std::cmp::Ordering;

pub trait Dictionary {
    fn contains(&self, s: &str) -> bool;
}

#[derive(Debug, Eq, PartialEq, Ord)]
pub struct Score {
    dictionary: usize,
    bak_penalty: bool,
    length: usize,
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use std::cmp::Ordering::*;
        
        match self.dictionary.cmp(&other.dictionary) {
            | result @ Greater 
            | result @ Less => return Some(result),

            _ => (),
        }

        match (self.bak_penalty, other.bak_penalty) {
            (true, false) => return Some(Less),
            (false, true) => return Some(Greater),

            _ => (),
        }

        Some(self.length.cmp(&other.length))
    }
}

#[derive(Debug)]
pub struct Ranker<D: Dictionary> {
    dictionary: D,
    nth_pattern: Regex,
}

impl<D: Dictionary> Ranker<D> {
    pub fn new(dictionary: D) -> Self {
        Self {
            dictionary,
            nth_pattern: Regex::new(r#".+ \(\d+\)"#).unwrap(),
        }
    }

    pub fn rank(&self, s: &str) -> Option<Score> {
        use std::path::Path;

        let path = Path::new(s);
        let name = path.file_name()?.to_str()?;
        let name = without_extension(name);

        Some(Score {
            dictionary: self.score(name),
            bak_penalty: self.has_penalty(name),
            length: name.len(),
        })
    }

    fn score(&self, name: &str) -> usize {
        name.split_whitespace()
            .filter(|s| self.dictionary.contains(s))
            .count()
    }

    fn has_penalty(&self, name: &str) -> bool {
        self.nth_pattern.is_match(name)
    }
}

fn without_extension(s: &str) -> &str {
    s.rfind('.').map(|idx| &s[..idx]).unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use {Dictionary, Ranker};
    use std::collections::HashSet;

    impl Dictionary for HashSet<String> {
        fn contains(&self, s: &str) -> bool {
            HashSet::contains(self, s)
        }
    }

    #[test]
    fn can_create_ranker() {
        build_ranker();
    }

    #[test]
    fn prefer_nonnumbered_names() {
        let ranker = build_ranker();
        let left = ranker.rank("/ordinary/name/number/one.txt");
        let right = ranker.rank("/ordinary/name/number/one (1).txt");

        assert!(left > right);
    }

    #[test]
    fn prefer_englishy_names() {
        let ranker = build_ranker();
        let left = ranker.rank("/ordinary/name/number/f87sf34UL.txt");
        let right = ranker.rank("/ordinary/name/number/one (1).txt");

        assert!(right > left);
    }

    fn build_ranker() -> Ranker<impl Dictionary> {
        let mut set = HashSet::new();
        set.insert(String::from("one"));
        Ranker::new(set)
    }
}
