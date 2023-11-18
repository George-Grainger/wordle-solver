use clap::{ArgEnum, Parser};
use wordle_solver::Guesser;

const GAMES: &str = include_str!("../answers.txt");

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the wordle guesser implementation to use
    #[clap(short, long, arg_enum)]
    implementation: Implementation,

    /// max Number of games to play
    #[clap(short, long)]
    max: Option<usize>,
}

/// various Worlde guesser implementations
#[derive(ArgEnum, Debug, Copy, Clone)]
enum Implementation {
    Unoptimised,
    Allocs,
    Vecrem,
}

fn main() {
    let args = Args::parse();
    match args.implementation {
        Implementation::Unoptimised => play(wordle_solver::algorithms::Unoptimised::new, args.max),
        Implementation::Allocs => play(wordle_solver::algorithms::Allocs::new, args.max),
        Implementation::Vecrem => play(wordle_solver::algorithms::Vecrem::new, args.max),
    }
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = wordle_solver::Wordle::new();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (mk)();
        if let Some(score) = w.play(answer, guesser) {
            println!("guessed '{}' in {}", &answer, score);
        } else {
            eprintln!("failed to guess.. exiting!");
        }
    }
}
