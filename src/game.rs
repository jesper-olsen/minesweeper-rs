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
    pub board: Vec<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
    pub num_mines: usize,
    pub game_state: GameState,
    first_click: bool,
    pub start_time: Option<Instant>,
    pub final_time: Option<Duration>,
}

impl Game {
    pub fn new(width: usize, height: usize, num_mines: usize) -> Self {
        let board = vec![
            vec![
                Cell {
                    content: CellContent::Number(0),
                    state: CellState::Covered,
                };
                width
            ];
            height
        ];

        Game {
            board,
            width,
            height,
            num_mines,
            game_state: GameState::Playing,
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
            self.board[*y][*x].content = CellContent::Mine;
        }

        self.calculate_numbers();
    }

    fn calculate_numbers(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.board[y][x].content == CellContent::Mine {
                    continue;
                }
                self.board[y][x].content = CellContent::Number(self.count_adjacent_mines(x, y));
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
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    if self.board[ny as usize][nx as usize].content == CellContent::Mine {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    pub fn reveal(&mut self, x: usize, y: usize) {
        if x >= self.width || y >= self.height || self.board[y][x].state != CellState::Covered {
            return;
        }

        if self.first_click {
            self.place_mines(x, y);
            self.first_click = false;
            self.start_time = Some(Instant::now());
        }

        self.board[y][x].state = CellState::Revealed;
        match self.board[y][x].content {
            CellContent::Mine => {
                self.game_state = GameState::Lost;
                self.board[y][x].content = CellContent::Explosion;
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
        if x < self.width && y < self.height && self.board[y][x].state != CellState::Revealed {
            self.board[y][x].state = match self.board[y][x].state {
                CellState::Covered => CellState::Flagged,
                CellState::Flagged => CellState::Covered,
                _ => self.board[y][x].state,
            };
        }
    }

    fn check_win_condition(&mut self) {
        let non_mine_cells = self.width * self.height - self.num_mines;
        let revealed_count = self
            .board
            .iter()
            .flatten()
            .filter(|c| c.state == CellState::Revealed)
            .count();
        if revealed_count == non_mine_cells {
            self.game_state = GameState::Won;
            if self.final_time.is_none() {
                if let Some(start) = self.start_time {
                    self.final_time = Some(start.elapsed());
                }
            }
        }
    }
}
