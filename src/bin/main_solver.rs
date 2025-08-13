use minesweeper_rs::{
    Difficulty,
    game::{CellState, Game, GameState},
};
use rand::Rng;
use rand::prelude::IndexedRandom;
use rayon::prelude::*;

fn benchmark_solver(num_games: usize, difficulty: Difficulty) -> usize {
    let (width, height, num_mines) = difficulty.dimensions();
    (0..num_games)
        .into_par_iter()
        .map(|_| {
            let mut rng = rand::rng();
            let mut game = Game::new(width, height, num_mines);

            // First click is random
            let first_x = rng.random_range(0..width);
            let first_y = rng.random_range(0..height);
            game.reveal(first_x, first_y);

            while game.state == GameState::Playing {
                let probs = game.calculate_all_bomb_probs();

                // Find lowest probability among covered cells
                let mut min_prob = f64::INFINITY;
                for y in 0..height {
                    for x in 0..width {
                        if game.get_cell(x, y).state == CellState::Covered {
                            let prob = probs[y * width + x];
                            if prob < min_prob {
                                min_prob = prob;
                            }
                        }
                    }
                }

                // Collect all cells with that min probability
                let mut candidates = Vec::new();
                for y in 0..height {
                    for x in 0..width {
                        if game.get_cell(x, y).state == CellState::Covered {
                            if (probs[y * width + x] - min_prob).abs() < 1e-12 {
                                candidates.push((x, y));
                            }
                        }
                    }
                }

                // Pick a random candidate
                if candidates.is_empty() {
                    break;
                }
                let &(xx, yy) = candidates.choose(&mut rng).unwrap();
                game.reveal(xx, yy);
            }

            (game.state == GameState::Won) as usize
        })
        .sum()
}

fn main() {
    let num_games = 1000;
    for difficulty in [
        Difficulty::Beginner,
        Difficulty::Intermediate,
        Difficulty::Expert,
    ] {
        let wins = benchmark_solver(num_games, difficulty);
        println!(
            "Difficulty {difficulty:?}: Solver won {}/{} games ({:.2}%)",
            wins,
            num_games,
            wins as f64 / num_games as f64 * 100.0
        );
    }

    //let (width, height, num_mines) = difficulty.dimensions();
}
