use crate::solver;
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
    pub start_time: Option<Instant>,
    pub final_time: Option<Duration>,
}

impl Game {
    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.board[y * self.width + x]
    }

    fn get_cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.board[y * self.width + x]
    }

    pub fn new(width: usize, height: usize, num_mines: usize) -> Self {
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
        }
    }

    fn place_mines(&mut self, avoid_x: usize, avoid_y: usize) {
        let mut rng = rand::rng();
        let mut possible_positions: Vec<(usize, usize)> = (0..self.height)
            .flat_map(|y| (0..self.width).map(move |x| (x, y)))
            .collect();

        // Remove the 3x3 area around the first click
        possible_positions.retain(|(x, y)| {
            !(((*x as isize - avoid_x as isize).abs() <= 1)
                && ((*y as isize - avoid_y as isize).abs() <= 1))
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
        if self.state != GameState::Playing {
            return 0.0;
        }

        let covered = self.count(CellState::Covered);
        let flagged = self.count(CellState::Flagged);
        let denom = covered + flagged;
        if denom == 0 {
            return 0.0;
        }
        let prior = self.num_mines as f64 / denom as f64;

        let n_cells = self.width * self.height;
        let mut p = vec![prior; n_cells];
        let mut q = vec![1.0 - prior; n_cells];
        let mut omega: Vec<(usize, Vec<usize>)> = Vec::new();
        let all_indices: Vec<usize> = (0..n_cells).collect();
        omega.push((self.num_mines, all_indices));

        for y in 0..self.height {
            for x in 0..self.width {
                let c = self.get_cell(x, y);
                if let CellContent::Number(n) = c.content {
                    let unrevealed = self.get_adjacent_unrevealed(x, y);
                    if !unrevealed.is_empty() {
                        omega.push((n as usize, unrevealed));
                    }
                }
            }
        }
        solver::solve_iterative_scaling(&mut p, &mut q, &omega, 50);

        let idx = cell_y * self.width + cell_x;
        p[idx]
    }
}
