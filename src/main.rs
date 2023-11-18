use clap::{ArgEnum, Parser};
use wordle_solver::{algorithms, Guesser};

const GAMES: &str = include_str!("../answers.txt");

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the wordle guesser implementation to use
    #[clap(short, long, arg_enum, default_value = "cutoff")]
    implementation: Implementation,

    /// max Number of games to play
    #[clap(short, long)]
    games: Option<usize>,
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
    Sigmoid,
    Cache,
}

fn main() {
    let args = Args::parse();
    match args.implementation {
        Implementation::Unoptimised => play::<algorithms::Unoptimised>(args.games),
        Implementation::Allocs => play::<algorithms::Allocs>(args.games),
        Implementation::Vecrem => play::<algorithms::Vecrem>(args.games),
        Implementation::Once => play::<algorithms::OnceInit>(args.games),
        Implementation::Precalc => play::<algorithms::Precalc>(args.games),
        Implementation::Weight => play::<algorithms::Weight>(args.games),
        Implementation::Cutoff => play::<algorithms::Cutoff>(args.games),
        Implementation::Enumerate => play::<algorithms::Enumerate>(args.games),
        Implementation::Popular => play::<algorithms::Popular>(args.games),
        Implementation::Sigmoid => play::<algorithms::Sigmoid>(args.games),
        Implementation::Cache => play::<algorithms::Cache>(args.games),
    }
}

fn play<G>(games: Option<usize>)
where
    G: Guesser + Default,
{
    let w = wordle_solver::Wordle::new();
    let mut score = 0;
    let mut played = 0;
    for answer in GAMES.split_whitespace().take(games.unwrap_or(usize::MAX)) {
        let guesser = G::default();
        if let Some(s) = w.play(answer, guesser) {
            played += 1;
            score += s;
            println!("guessed '{}' in {}", &answer, s);
        } else {
            eprintln!("failed to guess.. exiting!");
        }
    }
    println!("average score: {:.2}", score as f64 / played as f64);
}
