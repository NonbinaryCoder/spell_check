use std::fmt::Display;

pub fn compare(word: &str, reference: &str) -> WordScore {
    WordScore(distance::damerau_levenshtein(word, reference))
}

/// Opaque type representing score of word, lower is better
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct WordScore(usize);

impl Display for WordScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let score = self.0;
        write!(f, "Δ = {score}")
    }
}
