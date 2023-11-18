use crate::{enumerate_mask, Correctness, Guess, Guesser, DICTIONARY, MAX_MASK_ENUM};
use once_cell::sync::OnceCell;
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static str, usize)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

pub struct Cutoff {
    remaining: Cow<'static, Vec<(&'static str, usize)>>,
    patterns: Cow<'static, Vec<[Correctness; 5]>>,
}

impl Default for Cutoff {
    fn default() -> Self {
        Self::new()
    }
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
                    // TODO: apply sigmoid to counts
                    (word, count)
                }));
                words.sort_unstable_by_key(|&(_, count)| std::cmp::Reverse(count));
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
                );
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
        let stop = (self.remaining.len() / 3).max(20);
        for &(word, count_out) in &*self.remaining {
            // considering a world where we _did_ guess `word` and got `pattern` as the
            // correctness. now, compute what _then_ is left.

            // Rather than iterate over the patterns sequentially and add up the counts of words
            // that result in that pattern, we can instead keep a running total for each pattern
            // simultaneously by storing them in an array. We can do this since each candidate-word
            // pair deterministically produces only one mask.
            let mut totals = [0usize; MAX_MASK_ENUM];
            for (candidate, count) in &*self.remaining {
                let idx = enumerate_mask(&Correctness::compute(candidate, word));
                totals[idx] += count;
            }

            assert_eq!(totals.iter().sum::<usize>(), remaining_count, "{}", word);

            let sum: f64 = totals
                .into_iter()
                .filter(|t| *t != 0)
                .map(|t| {
                    // TODO: apply sigmoid
                    let p_of_this_pattern = t as f64 / remaining_count as f64;
                    p_of_this_pattern * p_of_this_pattern.log2()
                })
                .sum();

            let p_word = count_out as f64 / remaining_count as f64;
            let entropy = -sum;
            let goodness = p_word * entropy;

            if let Some(c) = best {
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness });
                }
            } else {
                best = Some(Candidate { word, goodness });
            }

            i += 1;
            if i >= stop {
                break;
            }
        }
        best.unwrap().word.to_string()
    }
}