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
    Once,
    Precalc,
    Weight,
    Cutoff,
    Enumerate,
    Popular,
}

fn main() {
    let args = Args::parse();
    match args.implementation {
        Implementation::Unoptimised => play(wordle_solver::algorithms::Unoptimised::new, args.max),
        Implementation::Allocs => play(wordle_solver::algorithms::Allocs::new, args.max),
        Implementation::Vecrem => play(wordle_solver::algorithms::Vecrem::new, args.max),
        Implementation::Once => play(wordle_solver::algorithms::OnceInit::new, args.max),
        Implementation::Precalc => play(wordle_solver::algorithms::Precalc::new, args.max),
        Implementation::Weight => play(wordle_solver::algorithms::Weight::new, args.max),
        Implementation::Cutoff => play(wordle_solver::algorithms::Cutoff::new, args.max),
        Implementation::Enumerate => play(wordle_solver::algorithms::Enumerate::new, args.max),
        Implementation::Popular => play(wordle_solver::algorithms::Popular::new, args.max),
    }
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = wordle_solver::Wordle::new();
    let mut score = 0;
    let mut games = 0;
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (mk)();
        if let Some(s) = w.play(answer, guesser) {
            games += 1;
            score += s;
            println!("guessed '{}' in {}", &answer, s);
        } else {
            eprintln!("failed to guess.. exiting!");
        }
    }
    println!("average score: {:.2}", score as f64 / games as f64);
}
