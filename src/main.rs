use clap::Parser;
use std::io::Result;
pub mod game;
pub mod tui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 18)]
    /// number of columns
    width: usize,

    #[arg(long, default_value_t = 10)]
    /// number of rows
    height: usize,

    #[arg(long, default_value_t = 25)]
    /// number of mines
    num_mines: usize,
}

// Levels in classic minesweeper (Windows 3.1/95):
// * Beginner 9 × 9, 10 mines (8.1%)
// * Intermediate 16 × 16, 40 mines (15.6%)
// * Expert	30 × 16, 99 mines (20.6%)
fn main() -> Result<()> {
    let args = Args::parse();

    if args.width * args.height <= args.num_mines {
        println!("Too many mines!");
        std::process::exit(0);
    }

    let game = game::Game::new(args.width, args.height, args.num_mines);
    let mut tui = tui::Tui::new(game)?;

    tui.game_loop()
}
