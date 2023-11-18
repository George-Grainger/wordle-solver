use crate::{Correctness, Guess, Guesser, DICTIONARY};
use once_cell::sync::OnceCell;
use std::{borrow::Cow, cmp::Reverse};

static INITIAL: OnceCell<Vec<(&'static str, usize)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

pub struct Cutoff {
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
    patterns: Cow<'static, Vec<[Correctness; 5]>>,
}

impl Cutoff {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                let mut words = Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("Every line is a word and a count");
                    let count: usize = count.parse().expect("every count is a number");
                    (word, count)
                }));
                words.sort_unstable_by_key(|&(_, count)| Reverse(count));
                words
            })),
            patterns: Cow::Borrowed(PATTERNS.get_or_init(|| Correctness::patterns().collect())),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static str,
    goodness: f64,
}

impl Guesser for Cutoff {
    fn guess(&mut self, history: &[Guess]) -> String {
        // Cutoff the dictionary by only keeping words that could be a possible match
        if let Some(last) = history.last() {
            if matches!(self.remaining, Cow::Owned(_)) {
                self.remaining
                    .to_mut()
                    .retain(|(word, _)| last.matches(word));
            } else {
                self.remaining = Cow::Owned(
                    self.remaining
                        .iter()
                        .filter(|(word, _)| last.matches(word))
                        .copied()
                        .collect(),
                )
            }
        }

        // hardcode the first guess to "tares"
        if history.is_empty() {
            self.patterns = Cow::Borrowed(PATTERNS.get().unwrap());
            return "tares".to_string();
        } else {
            assert!(!self.patterns.is_empty());
        }

        // the sum of the counts of all the remaining words in the dictionary
        let remaining_count: usize = self.remaining.iter().map(|(_, c)| c).sum();
        // the best word
        let mut best: Option<Candidate> = None;
        let mut i = 0;
        let stop = (self.remaining.len() / 3).max(16);
        for &(word, count_out) in &*self.remaining {
            let mut sum = 0.0;

            let check_pattern = |pattern: &[Correctness; 5]| {
                // total of the count(s) of words that match a pattern
                let mut in_pattern_total: usize = 0;

                // given a particular candidate word, if we guess this word, what
                // are the probabilities of getting each pattern. We sum together all those
                // probabilities and use that to determine the entropy information amount from
                // guessing that word
                let g = Guess {
                    word: Cow::Borrowed(word),
                    mask: *pattern,
                };
                for (candidate, count) in &*self.remaining {
                    // considering a "world" where we did guess "word" and got "pattern" as the
                    // correctness. Now compute what _then_ is left
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }
                if in_pattern_total == 0 {
                    return false;
                }
                let prob_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += prob_of_this_pattern * prob_of_this_pattern.log2();
                return true;
            };

            if matches!(self.patterns, Cow::Owned(_)) {
                self.patterns.to_mut().retain(check_pattern);
            } else {
                self.patterns = Cow::Owned(
                    self.patterns
                        .iter()
                        .copied()
                        .filter(check_pattern)
                        .collect(),
                );
            }

            let p_word = count_out as f64 / remaining_count as f64;
            let entropy = -sum;
            let goodness = p_word * entropy;

            if let Some(c) = best {
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness })
                }
            } else {
                best = Some(Candidate { word, goodness })
            }

            i += 1;
            if i > stop {
                break;
            }
        }
        best.unwrap().word.to_string()
    }
}
