use std::collections::HashSet;
use std::fmt;

use crate::{Constraint, FirstClickPolicy, solver};
use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

// --- Error types for robust error handling ---

#[derive(Debug)]
pub enum ParseGameError {
    /// The input string was empty or contained only whitespace.
    EmptyInput,
    /// The rows in the input string have inconsistent lengths.
    InconsistentRowLength {
        expected: usize,
        actual: usize,
        row_index: usize,
    },
}

impl fmt::Display for ParseGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseGameError::EmptyInput => write!(f, "Input text cannot be empty."),
            ParseGameError::InconsistentRowLength {
                expected,
                actual,
                row_index,
            } => write!(
                f,
                "Inconsistent row length at row {}: expected {}, but got {}",
                row_index, expected, actual
            ),
        }
    }
}

impl std::error::Error for ParseGameError {}

#[derive(Debug)]
pub enum LoadGameError {
    /// An error occurred while reading the file.
    Io(io::Error),
    /// An error occurred while parsing the file content.
    Parse(ParseGameError),
}

impl fmt::Display for LoadGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadGameError::Io(err) => write!(f, "I/O error: {}", err),
            LoadGameError::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl std::error::Error for LoadGameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadGameError::Io(err) => Some(err),
            LoadGameError::Parse(err) => Some(err),
        }
    }
}

// Automatically convert IO and Parse errors into LoadGameError
// This allows using the `?` operator for clean error handling.
impl From<io::Error> for LoadGameError {
    fn from(err: io::Error) -> Self {
        LoadGameError::Io(err)
    }
}

impl From<ParseGameError> for LoadGameError {
    fn from(err: ParseGameError) -> Self {
        LoadGameError::Parse(err)
    }
}

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
                    CellState::Covered if self.state == GameState::Playing => "#".to_string(),
                    CellState::Flagged if self.state == GameState::Playing => "F".to_string(),
                    _ => match cell.content {
                        CellContent::Mine => "*".to_string(),
                        CellContent::Explosion => "X".to_string(),
                        CellContent::Number(0) => ".".to_string(), // Dot for clarity on empty spaces
                        CellContent::Number(n) => n.to_string(),
                    },
                };
                write!(f, "{} ", representation)?; // space for padding
            }
            // Add a newline at the end of each row
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Game {
    /// Reads a file and uses `from_text` to parse its content into a Game.
    ///
    /// # Errors
    ///
    /// Returns a `LoadGameError` if the file cannot be read (`LoadGameError::Io`)
    /// or if the file content is not a valid grid (`LoadGameError::Parse`).
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, LoadGameError> {
        let content = fs::read_to_string(path)?;
        let game = Game::from_text(&content)?;
        Ok(game)
    }

    /// Creates a Game from a text representation of the minefield.
    ///
    /// The text should be a grid where '*' represents a mine and any other
    /// character represents a safe cell. All cells will be initialized in the
    /// 'Covered' state. The function will automatically calculate the numbers
    /// for the safe cells based on adjacent mines.
    ///
    /// # Errors
    ///
    /// This function will return an `Err` if the input text is not a valid grid
    /// (e.g., if rows have different lengths or the input is empty).
    ///
    /// # Example
    ///
    /// ```
    /// // Assuming this code is in a crate where Game is accessible
    /// use minesweeper_rs::game::Game;
    ///
    /// let board_layout = "
    /// .*.
    /// *..
    /// ...
    /// ";
    /// let board_layout = "
    /// ..*..
    /// .*...
    /// ..*..
    /// ";
    /// let game = Game::from_text(board_layout).unwrap(); // .unwrap() for example simplicity
    /// assert_eq!(game.width, 5);
    /// assert_eq!(game.height, 3);
    /// assert_eq!(game.num_mines, 3);
    /// ```
    pub fn from_text(text: &str) -> Result<Self, ParseGameError> {
        let lines: Vec<&str> = text
            .trim()
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        if lines.is_empty() {
            return Err(ParseGameError::EmptyInput);
        }

        let height = lines.len();
        let width = lines[0].chars().count();
        let mut num_mines = 0;
        let mut board = Vec::with_capacity(width * height);

        for (y, line) in lines.iter().enumerate() {
            let current_width = line.chars().count();
            if current_width != width {
                return Err(ParseGameError::InconsistentRowLength {
                    expected: width,
                    actual: current_width,
                    row_index: y,
                });
            }

            for char in line.chars() {
                let content = if char == '*' {
                    num_mines += 1;
                    CellContent::Mine
                } else {
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
            first_click: false, // normally mines are placed on first click
            first_click_policy: FirstClickPolicy::Unprotected,
            start_time: Some(Instant::now()),
            final_time: None,
        };

        game.calculate_numbers();

        Ok(game)
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.board[y * self.width + x]
    }

    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
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

    fn count_adjacent_revealed(&self, i: usize) -> usize {
        let x = i % self.width;
        let y = i / self.width;
        let mut n = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    let idx = (ny * self.width as isize + nx) as usize;
                    if self.board[idx].state == CellState::Revealed {
                        n += 1
                    }
                }
            }
        }
        n
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

    pub fn get_covered(&self) -> Vec<usize> {
        self.board
            .iter()
            .enumerate()
            .filter(|(_, c)| c.state != CellState::Revealed)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn get_sea_of_unknown(&self) -> Vec<usize> {
        (0..self.board.len())
            .map(|i| (i, self.count_adjacent_revealed(i) == 0))
            .filter(|(_, b)| *b)
            .map(|(i, _)| i)
            .collect()
    }

    /// returns a 3-tuple:
    /// * global constraint: all covered cell indicies and total num of mines
    /// * local constraints: list of cells and their mine count
    /// * sea_of_unknown: cell indices without local constraints
    pub fn get_constraints(&self) -> (Constraint, Vec<Constraint>, Vec<usize>) {
        let mut constraints_set: HashSet<Constraint> = HashSet::new();
        let unknown_indices: Vec<usize> = self.get_covered(); // all unknown
        let mut sea_of_unknown: Vec<usize> = self.get_sea_of_unknown(); // unknown wihout local constraints

        // Create the constraint for the total number of mines
        let global_constraint = Constraint::new(unknown_indices, self.num_mines as f64);

        // Add number constraints from unrevealed neighbours
        for y in 0..self.height {
            for x in 0..self.width {
                if let Cell {
                    content: CellContent::Number(n),
                    state: CellState::Revealed,
                } = *self.get_cell(x, y)
                {
                    let unrevealed = self.get_adjacent_unrevealed(x, y);
                    if !unrevealed.is_empty() {
                        // Create and insert the constraint. The HashSet handles duplicates.
                        constraints_set.insert(Constraint::new(unrevealed, n as f64));
                    }
                }
            }
        }
        sea_of_unknown.sort_unstable();
        // Convert the HashSet into a Vec for the return type
        (
            global_constraint,
            constraints_set.into_iter().collect(),
            sea_of_unknown,
        )
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

        let (global_constraint, mut local_constraints, _sea_of_unknown) = self.get_constraints();
        local_constraints.push(global_constraint);

        solver::solve_iterative_scaling(&mut p, &mut q, &local_constraints, 100);
        p
    }
}
