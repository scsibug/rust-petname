use rand::seq::SliceRandom;

/// Convenience function to generate a new petname from default word lists.
#[allow(dead_code)]
pub fn petname(words: u8, separator: &str) -> String {
    Petnames::default().generate_one(words, separator)
}

/// A word list.
pub type Words<'a> = Vec<&'a str>;

/// Word lists and the logic to combine them into _petnames_.
///
/// A _petname_ with `n` words will contain, in order:
///
///   * `n - 2` adverbs when `n >= 2`, otherwise 0 adverbs.
///   * 1 adjective when `n >= 2`, otherwise 0 adjectives.
///   * 1 name / noun when `n >= 1`, otherwise 0 names.
///
pub struct Petnames<'a> {
    pub adjectives: Words<'a>,
    pub adverbs: Words<'a>,
    pub names: Words<'a>,
}

impl<'a> Petnames<'a> {
    /// Constructs a new `Petnames` from the default (small) word lists.
    pub fn default() -> Self {
        Self::small()
    }

    /// Constructs a new `Petnames` from the small word lists.
    pub fn small() -> Self {
        Self::init(
            include_str!("../words/small/adjectives.txt"),
            include_str!("../words/small/adverbs.txt"),
            include_str!("../words/small/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the medium word lists.
    pub fn medium() -> Self {
        Self::init(
            include_str!("../words/medium/adjectives.txt"),
            include_str!("../words/medium/adverbs.txt"),
            include_str!("../words/medium/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the large word lists.
    pub fn large() -> Self {
        Self::init(
            include_str!("../words/large/adjectives.txt"),
            include_str!("../words/large/adverbs.txt"),
            include_str!("../words/large/names.txt"),
        )
    }

    /// Constructs a new `Petnames` from the given word lists.
    ///
    /// The words are extracted from the given strings by splitting on whitespace.
    pub fn init(adjectives: &'a str, adverbs: &'a str, names: &'a str) -> Self {
        Self {
            adjectives: adjectives.split_whitespace().collect(),
            adverbs: adverbs.split_whitespace().collect(),
            names: names.split_whitespace().collect(),
        }
    }

    /// Keep words matching a predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut petnames = petname::Petnames::default();
    /// petnames.retain(|s| s.starts_with("b"));
    /// petnames.generate_one(2, ".");
    /// ```
    ///
    /// This is merely a convenience wrapper that applies the same predicate to
    /// the adjectives, adverbs, and names lists.
    ///
    pub fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&&str) -> bool,
    {
        self.adjectives.retain(&predicate);
        self.adverbs.retain(&predicate);
        self.names.retain(&predicate);
    }

    /// Calculate the cardinality of this `Petnames`.
    ///
    /// If this is low, names may be repeated by the generator with a higher
    /// frequency than your use-case may allow. If it is 0 (zero) the generator
    /// will panic (unless `words` is also zero).
    ///
    /// This can saturate. If the total possible combinations of words exceeds
    /// `u128::MAX` then this will return `u128::MAX`.
    pub fn cardinality(&self, words: u8) -> u128 {
        let init: u128 = if words == 0 { 0 } else { 1 };
        Lists(self, words)
            .map(|list| list.len() as u128)
            .fold(init, |acc, len| acc.saturating_mul(len))
    }

    /// Generate a new petname.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut rng = rand::thread_rng();
    /// petname::Petnames::default().generate(&mut rng, 7, ":");
    /// ```
    ///
    pub fn generate<RNG>(&self, rng: &mut RNG, words: u8, separator: &str) -> String
    where
        RNG: rand::Rng,
    {
        Lists(self, words)
            .filter_map(|list| list.choose(rng))
            .cloned()
            .collect::<Vec<&str>>()
            .join(separator)
    }

    /// Generate a single new petname.
    ///
    /// This is like `generate` but uses `rand::thread_rng` as the random
    /// source. For efficiency use `generate` when creating multiple names, or
    /// when you want to use a custom source of randomness.
    pub fn generate_one(&self, words: u8, separator: &str) -> String {
        self.generate(&mut rand::thread_rng(), words, separator)
    }
}

impl<'a> Default for Petnames<'a> {
    fn default() -> Self {
        Self::default()
    }
}

/// Iterator over a `Petnames`' word lists.
///
/// This yields the appropriate lists from which to select a word when
/// constructing a petname of `n` words. For example, if you want 3 words in
/// your petname, this will first yield the adverbs word list, then adjectives,
/// then names.
struct Lists<'a>(&'a Petnames<'a>, u8);

impl<'a> Iterator for Lists<'a> {
    type Item = &'a Words<'a>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.1 as usize))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let Self(petnames, ref mut word) = self;
        match word {
            0 => None,
            1 => {
                *word -= 1;
                Some(&petnames.names)
            }
            2 => {
                *word -= 1;
                Some(&petnames.adjectives)
            }
            _ => {
                *word -= 1;
                Some(&petnames.adverbs)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{petname, Petnames};
    use rand;

    #[test]
    fn default_petnames_has_adjectives() {
        let petnames = Petnames::default();
        assert_ne!(petnames.adjectives.len(), 0);
    }

    #[test]
    fn default_petnames_has_adverbs() {
        let petnames = Petnames::default();
        assert_ne!(petnames.adverbs.len(), 0);
    }

    #[test]
    fn default_petnames_has_names() {
        let petnames = Petnames::default();
        assert_ne!(petnames.names.len(), 0);
    }

    #[test]
    fn default_petnames_has_non_zero_cardinality() {
        let petnames = Petnames::default();
        // This test will need to be adjusted when word lists change.
        assert_eq!(0, petnames.cardinality(0));
        assert_eq!(456, petnames.cardinality(1));
        assert_eq!(204744, petnames.cardinality(2));
        assert_eq!(53438184, petnames.cardinality(3));
        assert_eq!(13947366024, petnames.cardinality(4));
    }

    #[test]
    fn generate_uses_adverb_adjective_name() {
        let petnames = Petnames {
            adjectives: vec!["adjective"],
            adverbs: vec!["adverb"],
            names: vec!["name"],
        };
        assert_eq!(
            petnames.generate(&mut rand::thread_rng(), 3, "-"),
            "adverb-adjective-name"
        );
    }

    #[test]
    fn petname_renders_desired_number_of_words() {
        assert_eq!(petname(7, "-").split("-").count(), 7);
    }

    #[test]
    fn petname_renders_with_desired_separator() {
        assert_eq!(petname(7, "@").split("@").count(), 7);
    }
}
