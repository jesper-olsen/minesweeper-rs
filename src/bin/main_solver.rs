use minesweeper_rs::{
    Difficulty, FirstClickPolicy,
    game::{CellState, Game, GameState},
};
use rand::Rng;
use rand::prelude::IndexedRandom;
use rayon::prelude::*;

fn benchmark_solver(
    num_games: usize,
    difficulty: Difficulty,
    first_click_policy: FirstClickPolicy,
    first_click: Option<(usize, usize)>,
) -> usize {
    let (width, height, num_mines) = difficulty.dimensions();
    (0..num_games)
        .into_par_iter()
        .map(|_| {
            let mut rng = rand::rng();
            let mut game = Game::new(width, height, num_mines, first_click_policy);

            // Use provided coordinate or generate random one
            let (first_x, first_y) = first_click
                .unwrap_or_else(|| (rng.random_range(0..width), rng.random_range(0..height)));
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

// fn bench_random() {
//     let num_games = 1000;
//     let first_click_policy = FirstClickPolicy::Unprotected;
//     //let first_click_policy = FirstClickPolicy::GuaranteedZero;
//     for difficulty in [
//         Difficulty::Beginner,
//         Difficulty::Intermediate,
//         Difficulty::Expert,
//     ] {
//         let first_click = None;
//         let wins = benchmark_solver(num_games, difficulty, first_click_policy, first);
//         println!(
//             "Difficulty {difficulty:?}: Solver won {}/{} games ({:.2}%)",
//             wins,
//             num_games,
//             wins as f64 / num_games as f64 * 100.0
//         );
//     }
// }

fn heatmap() {
    let num_games = 10000;
    let first_click_policy = FirstClickPolicy::Unprotected;
    //let first_click_policy = FirstClickPolicy::GuaranteedZero;
    //let first_click_policy = FirstClickPolicy::GuaranteedSafe;
    //let difficulty = Difficulty::Intermediate;
    //let difficulty = Difficulty::Expert;
    let difficulty = Difficulty::Beginner;
    let (width, height, _) = difficulty.dimensions();

    // Output for plotting
    for y in (0..height).rev() {
        for x in 0..width {
            let first_click = Some((x, y));
            let wins = benchmark_solver(num_games, difficulty, first_click_policy, first_click);
            let win_rate = wins as f64 / num_games as f64 * 100.0;
            // space between values, no trailing space at end of line
            print!("{win_rate:.2} ");
        }
        println!();
    }
}

fn main() {
    heatmap();
}
