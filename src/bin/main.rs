use clap::Parser;
use minesweeper_rs::{
    Difficulty, {game, tui},
};
use std::io::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum)]
    /// Use a classic difficulty preset (overrides width/height/mines)
    difficulty: Option<Difficulty>,

    #[arg(long, default_value_t = 9)]
    /// Number of columns (ignored if difficulty is set)
    width: usize,

    #[arg(long, default_value_t = 9)]
    /// Number of rows (ignored if difficulty is set)
    height: usize,

    #[arg(long, default_value_t = 10)]
    /// Number of mines (ignored if difficulty is set)
    num_mines: usize,

    #[arg(long)]
    /// List available difficulty presets and exit
    list_difficulties: bool,

    #[arg(long, default_value_t = false)]
    /// display bomb probabilities - in the status bar for cell under the cursor.
    display_bomb_prob: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.list_difficulties {
        println!("Available difficulties:");
        println!("  beginner     - 9x9, 10 mines (8%)");
        println!("  intermediate - 16x16, 40 mines (16%)");
        println!("  expert       - 30x16, 99 mines (21%)");
        std::process::exit(0);
    }

    let (width, height, num_mines) = if let Some(difficulty) = args.difficulty {
        difficulty.dimensions()
    } else {
        (args.width, args.height, args.num_mines)
    };

    if width * height <= num_mines + 9 {
        println!(
            "Error: Too many mines! Need at least {min_cells} cells for {num_mines} mines (including 9 mine-free cells around first click).",
            min_cells = num_mines + 10
        );
        std::process::exit(1);
    }

    let game = game::Game::new(width, height, num_mines);
    let mut tui = tui::Tui::new(game, args.display_bomb_prob)?;

    tui.game_loop()
}
