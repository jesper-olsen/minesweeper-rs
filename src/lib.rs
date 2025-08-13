pub mod game;
pub mod solver;
pub mod tui;

use clap::ValueEnum;

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
}

impl Difficulty {
    pub fn dimensions(&self) -> (usize, usize, usize) {
        match self {
            Difficulty::Beginner => (9, 9, 10),
            Difficulty::Intermediate => (16, 16, 40),
            Difficulty::Expert => (30, 16, 99),
        }
    }
}
