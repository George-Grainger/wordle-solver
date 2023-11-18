use std::{borrow::Cow, collections::HashSet};

pub mod algorithms;

const DICTIONARY: &str = include_str!("../dictionary.txt");

pub struct Wordle {
    dictionary: HashSet<&'static str>,
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("Every line is a word and a count")
                    .0
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {
        // Play six rounders where it invokes guesser each round
        let mut history = Vec::new();
        // Wordle allows six guesses.
        // We allow more to avoid chopping off the score distribution for stats purposes.
        for i in 1..=32 {
            let guess = guesser.guess(&history);
            assert!(
                self.dictionary.contains(&*guess),
                "guess '{}' isn't in the dictionary",
                guess
            );
            if guess == answer {
                return Some(i);
            }

            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: Cow::Owned(guess),
                mask: correctness,
            });
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Correctness {
    /// Green
    Correct,
    /// Yellow
    Misplaced,
    /// Grey
    Wrong,
}

impl Correctness {
    fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        let mut c = [Correctness::Wrong; 5];
        let answer_bytes = answer.as_bytes();
        let guess_bytes = guess.as_bytes();

        for (i, &a) in answer_bytes.iter().enumerate() {
            if a == guess_bytes[i] {
                // Mark things green
                c[i] = Correctness::Correct;
            } else if let Some(j) = guess_bytes.iter().enumerate().position(|(j, &g)| {
                // The position in guess can only be marked as Misplaced, if it isn't Correct and wasn't already marked before.
                a == g && answer_bytes[j] != a && c[j] == Correctness::Wrong
            }) {
                // Mark things yellow
                c[j] = Correctness::Misplaced;
            }
        }

        c
    }

    /// computes the Cartesian Product of all possible correctness patterns for a 5 letter word.
    /// returns an Iterator over an array containing a possible pattern
    ///
    /// There are 3 correctness patterns for each of the 5 character positions in a word, so the
    /// total patterns will be of length 3^5.
    /// Some patterns are impossible to reach so in reality this would be slightly
    /// less than 3^5, but it should not affect our calculations. We'll generate the Cartesian
    /// Product and optimize later
    pub fn patterns() -> impl Iterator<Item = [Self; 5]> {
        itertools::iproduct!(
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong]
        )
        .map(|(a, b, c, d, e)| [a, b, c, d, e])
    }
}

pub struct Guess<'a> {
    pub word: Cow<'a, str>,
    pub mask: [Correctness; 5],
}

impl Guess<'_> {
    pub fn matches(&self, word: &str) -> bool {
        // If guess G gives mask C against answer A, then
        // guess A should also give mask C against answer G
        Correctness::compute(word, &self.word) == self.mask
    }
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
}

impl Guesser for fn(history: &[Guess]) -> String {
    fn guess(&mut self, history: &[Guess]) -> String {
        (*self)(history)
    }
}

/// helper macro that returns a struct implementing the Guesser trait.
/// It allows you to pass in a closure that can be used to mock the results of the guess fn
///
/// # Example
/// `guesser!(|_history| { "moved".to_string() });`
#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl $crate::Guesser for G {
            fn guess(&mut self, $history: &[Guess]) -> String {
                $impl
            }
        }
        G
    }};
}

/// maps a list of C,M,W tokens into an array of Correctness variants
#[cfg(test)]
macro_rules! mask {
    (C) => { $crate::Correctness::Correct };
    (M) => { $crate::Correctness::Misplaced };
    (W) => { $crate::Correctness::Wrong };
    ($($c:tt)+) => {[
        $(mask!($c)),+
    ]}
}

#[cfg(test)]
mod tests {

    mod guess_matcher {
        use crate::Guess;
        use std::borrow::Cow;

        /// checks if a Guess matches a word
        /// Ex. `check!("abcde" + [C C C C C] allows "abcde");`
        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                assert!(Guess {
                    word: Cow::Borrowed($prev),
                    mask: mask![$($mask )+]
                }.matches($next));
            };
            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                assert!(!Guess {
                    word: Cow::Borrowed($prev),
                    mask: mask![$($mask )+]
                }.matches($next));
            }
        }

        #[test]
        fn matches() {
            // checking previous guess + prev. mask, against the latest guessed word
            check!("abcde" + [C C C C C] allows "abcde");
            check!("abcdf" + [C C C C C] disallows "abcde");
            check!("abcde" + [W W W W W] allows "fghij");
            check!("abcde" + [M M M M M] allows "eabcd");
            check!("baaaa" + [W C M W W] allows "aaccc");
            check!("baaaa" + [W C M W W] disallows "caacc");
            check!("aaabb" + [C M W W W] disallows "accaa");
            check!("tares" + [W M M W W] disallows "brink");
        }
    }

    mod game {
        use crate::{Guess, Wordle};

        #[test]
        fn play_first_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "right".to_string() });
            assert_eq!(w.play("right", guesser), Some(1));
        }

        #[test]
        fn play_second_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(2));
        }

        #[test]
        fn play_third_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(3));
        }

        #[test]
        fn play_fourth_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(4));
        }

        #[test]
        fn play_fifth_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(5));
        }

        #[test]
        fn play_sixth_guess_is_correct() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });

            assert_eq!(w.play("right", guesser), Some(6));
        }

        #[test]
        fn all_wrong_guesses_should_terminate() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "wrong".to_string() });

            assert_eq!(w.play("right", guesser), None);
        }
    }

    mod compute {
        use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask!(C C C C C))
        }

        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute("abcde", "qwxyz"), mask!(W W W W W))
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcde", "eabcd"), mask!(M M M M M))
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask!(C C W W W))
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask!(W W M M W))
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute("aabbb", "caacc"), mask!(W C M W W))
        }

        #[test]
        fn some_green_some_yellow() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask!(C M W W W))
        }

        #[test]
        fn one_green() {
            assert_eq!(Correctness::compute("baccc", "aaddd"), mask!(W C W W W))
        }

        #[test]
        fn some_green_some_yellow2() {
            assert_eq!(Correctness::compute("abcde", "aacde"), mask!(C W C C C))
        }
    }
}
