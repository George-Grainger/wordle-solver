const GAMES: &str = include_str!("../answers.txt");

fn main() {
    let w = wordle_solver::Wordle::new();
    for answer in GAMES.split_whitespace() {
        let guesser = wordle_solver::algorithms::Naive::new();
        w.play(answer, guesser);
    }
    println!("Hello, world!");
}
