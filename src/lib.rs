use std::collections::HashSet;

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
                word: guess,
                mask: correctness,
            });
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correctness {
    /// Green
    Correct,
    /// Yellow
    Misplaced,
    /// Missing
    Wrong,
}

impl Correctness {
    fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);

        let mut c = [Correctness::Wrong; 5];

        // Mark things green
        for (i, (a, g)) in answer.chars().zip(guess.chars()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
            }
        }

        // Mark things yellow
        let mut used = [false; 5];
        for (i, &c) in c.iter().enumerate() {
            if c == Correctness::Correct {
                used[i] = true;
            }
        }
        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                // Already marked as green
                continue;
            }
            if answer.chars().enumerate().any(|(j, a)| {
                if a == g && !used[j] {
                    used[j] = true;
                    return true;
                }
                false
            }) {
                c[i] = Correctness::Misplaced;
            }
        }
        c
    }
}

pub struct Guess {
    pub word: String,
    pub mask: [Correctness; 5],
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

#[cfg(test)]
mod tests {
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
            let guesser = guesser!(|history| { "wrong".to_string() });

            assert_eq!(w.play("right", guesser), None);
        }
    }

    mod compute {
        use crate::Correctness;

        macro_rules! mask {
            (C) => {
                Correctness::Correct
            };
            (M) => {
                Correctness::Misplaced
            };
            (W) => {
                Correctness::Wrong
            };
            ($($c:tt)+) => {[
                $(mask!($c)),+
            ]};
        }

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
