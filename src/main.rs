use clap::{ArgEnum, Parser};
use wordle_solver::{algorithms, Guesser};

const GAMES: &str = include_str!("../answers.txt");

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, arg_enum, default_value = "cache")]
    implementation: Implementation,

    #[clap(short, long)]
    games: Option<usize>,
}

#[derive(ArgEnum, Debug, Clone, Copy)]
enum Implementation {
    Unoptimised,
    Allocs,
    Vecrem,
    Precalc,
    Weight,
    Enum,
    Cutoff,
    Sigmoid,
    Escore,
    Popular,
    Cache,
}

fn main() {
    let args = Args::parse();

    match args.implementation {
        Implementation::Unoptimised => play::<algorithms::Unoptimised>(args.games),
        Implementation::Allocs => play::<algorithms::Allocs>(args.games),
        Implementation::Vecrem => play::<algorithms::Vecrem>(args.games),
        Implementation::Precalc => play::<algorithms::Precalc>(args.games),
        Implementation::Weight => play::<algorithms::Weight>(args.games),
        Implementation::Enum => play::<algorithms::Enumerate>(args.games),
        Implementation::Cutoff => play::<algorithms::Cutoff>(args.games),
        Implementation::Sigmoid => play::<algorithms::Sigmoid>(args.games),
        Implementation::Escore => play::<algorithms::Escore>(args.games),
        Implementation::Popular => play::<algorithms::Popular>(args.games),
        Implementation::Cache => play::<algorithms::Cached>(args.games),
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
}
