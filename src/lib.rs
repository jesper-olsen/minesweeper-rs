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

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum FirstClickPolicy {
    GuaranteedZero, // 0-cell (3x3 opening)
    GuaranteedSafe, // mine free
    Unprotected,    // can hit a mine
}

// #[derive(Debug)]
// pub struct Constraint {
//     pub cells: Vec<usize>, // cell indexes
//     pub count: f64,        // can be integer-like or fractional
// }

// impl Constraint {
//     pub fn new(cells: Vec<usize>, count: impl Into<f64>) -> Self {
//         Self {
//             cells,
//             count: count.into(),
//         }
//     }
// }

use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Constraint {
    pub cells: Vec<usize>, // cell indexes
    pub count: f64,        // can be integer-like or fractional
}

impl Constraint {
    /// Creates a new Constraint, ensuring cells are sorted to maintain a
    /// canonical representation for hashing and equality checks.
    pub fn new(mut cells: Vec<usize>, count: impl Into<f64>) -> Self {
        // Sort the cells to create a canonical form. This is CRITICAL.
        cells.sort_unstable();
        Self {
            cells,
            count: count.into(),
        }
    }
}

// Manual implementation of PartialEq
impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        // Compare the sorted cell vectors directly.
        // For floats, compare their bit patterns for exact equality.
        // This correctly handles cases like +0.0 and -0.0 being different.
        self.cells == other.cells && self.count.to_bits() == other.count.to_bits()
    }
}

// Since we've defined a robust PartialEq that satisfies the rules
// (e.g., a == a), we can now declare that our type implements Eq.
impl Eq for Constraint {}

// Manual implementation of Hash - because count is f64
impl Hash for Constraint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the components of the struct.
        self.cells.hash(state);
        self.count.to_bits().hash(state); // Hash the integer bit pattern of the float
    }
}
