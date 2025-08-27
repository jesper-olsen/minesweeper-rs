use std::fmt;

use crate::{
    FirstClickPolicy,
    solver::{self, Constraint},
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellContent {
    Mine,
    Explosion,
    Number(u8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellState {
    Covered,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub content: CellContent,
    pub state: CellState,
}

#[derive(Debug, PartialEq)]
pub enum GameState {
    Playing,
    Won,
    Lost,
}

pub struct Game {
    board: Vec<Cell>,
    pub width: usize,
    pub height: usize,
    pub num_mines: usize,
    pub state: GameState,
    first_click: bool,
    pub first_click_policy: FirstClickPolicy,
    pub start_time: Option<Instant>,
    pub final_time: Option<Duration>,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.get_cell(x, y);
                let representation = match cell.state {
                    CellState::Covered => "#".to_string(),
                    CellState::Flagged => "F".to_string(),
                    CellState::Revealed => match cell.content {
                        CellContent::Mine => "*".to_string(), // Should not be revealed unless game is over
                        CellContent::Explosion => "X".to_string(),
                        CellContent::Number(0) => ".".to_string(), // Dot for clarity on empty spaces
                        CellContent::Number(n) => n.to_string(),
                    },
                };
                // Write the character with a space for padding
                write!(f, "{} ", representation)?;
            }
            // Add a newline at the end of each row
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Game {
    /// Creates a Game from a text representation of the minefield.
    ///
    /// The text should be a grid where '*' represents a mine and any other
    /// character represents a safe cell. All cells will be initialized in the
    /// 'Covered' state. The function will automatically calculate the numbers
    /// for the safe cells based on adjacent mines.
    ///
    /// # Panics
    ///
    /// This function will panic if the input text is not a valid grid (e.g.,
    /// if rows have different lengths or the input is empty after trimming).
    ///
    /// # Example
    ///
    /// ```
    /// // Assuming this code is in a crate where Game is accessible
    /// use minesweeper_rs::game::Game;
    ///
    /// let board_layout = "
    /// ..*..
    /// .*...
    /// ..*..
    /// ";
    /// let game = Game::from_text(board_layout);
    /// assert_eq!(game.width, 5);
    /// assert_eq!(game.height, 3);
    /// assert_eq!(game.num_mines, 3);
    /// ```
    pub fn from_text(text: &str) -> Self {
        let lines: Vec<&str> = text
            .trim()
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        if lines.is_empty() {
            // Return an empty game for empty input
            return Game::new(0, 0, 0, FirstClickPolicy::Unprotected);
        }

        let height = lines.len();
        let width = lines[0].chars().count();
        let mut num_mines = 0;
        let mut board = Vec::with_capacity(width * height);

        for line in &lines {
            // Ensure all lines have the same width for a valid grid
            assert_eq!(
                line.chars().count(),
                width,
                "All rows in the input text must have the same length."
            );

            for char in line.chars() {
                let content = if char == '*' {
                    num_mines += 1;
                    CellContent::Mine
                } else {
                    // Placeholder for now, will be calculated later
                    CellContent::Number(0)
                };
                board.push(Cell {
                    content,
                    state: CellState::Covered,
                });
            }
        }

        let mut game = Game {
            board,
            width,
            height,
            num_mines,
            state: GameState::Playing,
            first_click: true,
            first_click_policy: FirstClickPolicy::Unprotected,
            start_time: None,
            final_time: None,
        };

        game.calculate_numbers();

        game
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.board[y * self.width + x]
    }

    fn get_cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.board[y * self.width + x]
    }

    pub fn new(
        width: usize,
        height: usize,
        num_mines: usize,
        first_click_policy: FirstClickPolicy,
    ) -> Self {
        let board = vec![
            Cell {
                content: CellContent::Number(0),
                state: CellState::Covered,
            };
            width * height
        ];

        Game {
            board,
            width,
            height,
            num_mines,
            state: GameState::Playing,
            first_click: true,
            start_time: None,
            final_time: None,
            first_click_policy,
        }
    }

    fn place_mines(&mut self, first_x: usize, first_y: usize) {
        let mut rng = rand::rng();
        let mut possible_positions: Vec<(usize, usize)> = (0..self.height)
            .flat_map(|y| (0..self.width).map(move |x| (x, y)))
            .collect();

        let avoid_width = match self.first_click_policy {
            FirstClickPolicy::GuaranteedZero => 1,
            FirstClickPolicy::GuaranteedSafe => 0,
            FirstClickPolicy::Unprotected => -1,
        };

        // Remove the 3x3 area around the first click
        possible_positions.retain(|(x, y)| {
            !(((*x as isize - first_x as isize).abs() <= avoid_width)
                && ((*y as isize - first_y as isize).abs() <= avoid_width))
        });

        // Shuffle the valid positions
        use rand::seq::SliceRandom;
        possible_positions.shuffle(&mut rng);

        // Take the required number of mines from the shuffled list
        for (x, y) in possible_positions.iter().take(self.num_mines) {
            self.get_cell_mut(*x, *y).content = CellContent::Mine;
        }

        self.calculate_numbers();
    }

    fn calculate_numbers(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_cell(x, y).content != CellContent::Mine {
                    let n = self.count_adjacent_mines(x, y);
                    self.get_cell_mut(x, y).content = CellContent::Number(n);
                }
            }
        }
    }

    fn count_adjacent_mines(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                if nx >= 0
                    && nx < self.width as isize
                    && ny >= 0
                    && ny < self.height as isize
                    && self.get_cell(nx as usize, ny as usize).content == CellContent::Mine
                {
                    count += 1;
                }
            }
        }
        count
    }

    // returns adjacent cell indices for unrevealed states
    fn get_adjacent_unrevealed(&self, x: usize, y: usize) -> Vec<usize> {
        let mut adjacent = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    let idx = (ny * self.width as isize + nx) as usize;
                    if self.board[idx].state != CellState::Revealed {
                        adjacent.push(idx);
                    }
                }
            }
        }
        adjacent
    }

    pub fn reveal(&mut self, x: usize, y: usize) {
        if x >= self.width || y >= self.height || self.get_cell(x, y).state != CellState::Covered {
            return;
        }

        if self.first_click {
            self.place_mines(x, y);
            self.first_click = false;
            self.start_time = Some(Instant::now());
        }

        self.get_cell_mut(x, y).state = CellState::Revealed;
        match self.get_cell(x, y).content {
            CellContent::Mine => {
                self.state = GameState::Lost;
                self.get_cell_mut(x, y).content = CellContent::Explosion;
                if let Some(start) = self.start_time {
                    self.final_time = Some(start.elapsed());
                }
            }
            CellContent::Number(0) => {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x as isize + dx, y as isize + dy);
                        if nx >= 0
                            && nx < self.width as isize
                            && ny >= 0
                            && ny < self.height as isize
                        {
                            self.reveal(nx as usize, ny as usize);
                        }
                    }
                }
            }
            _ => {}
        }
        self.check_win_condition();
    }

    pub fn flag(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height && self.get_cell(x, y).state != CellState::Revealed {
            self.get_cell_mut(x, y).state = match self.get_cell(x, y).state {
                CellState::Covered => CellState::Flagged,
                CellState::Flagged => CellState::Covered,
                _ => self.get_cell(x, y).state,
            };
        }
    }

    pub fn count(&self, cell_state: CellState) -> usize {
        self.board.iter().filter(|c| c.state == cell_state).count()
    }

    fn check_win_condition(&mut self) {
        let non_mine_cells = self.width * self.height - self.num_mines;
        if self.count(CellState::Revealed) == non_mine_cells {
            self.state = GameState::Won;
            if self.final_time.is_none() {
                if let Some(start) = self.start_time {
                    self.final_time = Some(start.elapsed());
                }
            }
        }
    }

    pub fn get_bomb_prob(&self, cell_x: usize, cell_y: usize) -> f64 {
        if self.get_cell(cell_x, cell_y).state == CellState::Revealed {
            return 0.0;
        }
        let p = self.calculate_all_bomb_probs();
        let idx = cell_y * self.width + cell_x;
        p[idx]
    }

    pub fn calculate_all_bomb_probs(&self) -> Vec<f64> {
        let n_cells = self.width * self.height;
        if self.state != GameState::Playing {
            return vec![0.0; n_cells];
        }

        let covered = self.count(CellState::Covered);
        let flagged = self.count(CellState::Flagged);
        let denom = covered + flagged;
        if denom == 0 {
            return vec![0.0; n_cells];
        }
        let prior = self.num_mines as f64 / denom as f64;

        let mut p = vec![prior; n_cells];
        let mut q = vec![1.0 - prior; n_cells];
        for i in 0..n_cells {
            if self.board[i].state == CellState::Revealed {
                p[i] = 0.0;
                q[i] = 1.0;
            }
        }
        let unknown_indices: Vec<usize> = (0..n_cells)
            .filter(|&i| self.board[i].state != CellState::Revealed)
            .collect();
        let mut constraints = Vec::new();
        constraints.push(Constraint::new(unknown_indices, self.num_mines as f64));

        // add number constraints - from unrevealed neighbours
        for y in 0..self.height {
            for x in 0..self.width {
                if let Cell {
                    content: CellContent::Number(n),
                    state: CellState::Revealed,
                } = *self.get_cell(x, y)
                {
                    let unrevealed = self.get_adjacent_unrevealed(x, y);
                    if !unrevealed.is_empty() {
                        constraints.push(Constraint::new(unrevealed, n));
                    }
                }
            }
        }

        solver::solve_iterative_scaling(&mut p, &mut q, &constraints, 100);
        p
    }
}
