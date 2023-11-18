use clap::{ArgEnum, Parser};
use wordle_solver::{algorithms, Guesser};

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
        Implementation::Unoptimised => play::<algorithms::Unoptimised>(args.max),
        Implementation::Allocs => play::<algorithms::Allocs>(args.max),
        Implementation::Vecrem => play::<algorithms::Vecrem>(args.max),
        Implementation::Once => play::<algorithms::OnceInit>(args.max),
        Implementation::Precalc => play::<algorithms::Precalc>(args.max),
        Implementation::Weight => play::<algorithms::Weight>(args.max),
        Implementation::Cutoff => play::<algorithms::Cutoff>(args.max),
        Implementation::Enumerate => play::<algorithms::Enumerate>(args.max),
        Implementation::Popular => play::<algorithms::Popular>(args.max),
    }
}

fn play<G>(max: Option<usize>)
where
    G: Guesser + Default,
{
    let w = wordle_solver::Wordle::new();
    let mut score = 0;
    let mut games = 0;
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = G::default();
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
