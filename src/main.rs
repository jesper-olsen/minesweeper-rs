use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use rand::Rng;
use std::io::{self, Result, Write};
use std::time::{Duration, Instant};

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

// --- CONFIGURATION & SYMBOLS ---
const CELL_WIDTH: u16 = 3; // Each cell will be 3 characters wide
const CURSOR_BG_COLOR: Color = Color::DarkYellow;

// Use simple, single-width ASCII characters. They will be padded.
const BOMB: char = '💣';
const FLAG: char = '🚩';
const EXPLOSION: char = '💥';
const COVERED: char = '#';
const EMPTY: char = '.';

// Offsets for drawing the board on the screen
const BOARD_OFFSET_X: u16 = 3;
const BOARD_OFFSET_Y: u16 = 5;

#[derive(Clone, Copy, Debug, PartialEq)]
enum CellContent {
    Mine,
    Explosion,
    Number(u8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CellState {
    Covered,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy, Debug)]
struct Cell {
    content: CellContent,
    state: CellState,
}

#[derive(Debug, PartialEq)]
enum GameState {
    Playing,
    Won,
    Lost,
}

struct Game {
    board: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
    num_mines: usize,
    cursor_x: usize, // TODO: move to Tui?
    cursor_y: usize,
    game_state: GameState,
    first_click: bool,
    start_time: Option<Instant>, // TODO: move to Tui?
    final_time: Option<Duration>,
}

impl Game {
    fn new(width: usize, height: usize, num_mines: usize) -> Self {
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
            cursor_x: width / 2,
            cursor_y: height / 2,
            game_state: GameState::Playing,
            first_click: true,
            start_time: None,
            final_time: None,
        }
    }

    fn place_mines(&mut self, avoid_x: usize, avoid_y: usize) {
        let mut rng = rand::rng();
        let mut mines_placed = 0;
        // TODO: return Err if fails to place mines
        while mines_placed < self.num_mines {
            let x = rng.random_range(0..self.width);
            let y = rng.random_range(0..self.height);

            // no mines placed near first square
            if (x as isize - avoid_x as isize).abs() <= 1
                && (y as isize - avoid_y as isize).abs() <= 1
            {
                continue;
            }

            if self.board[y][x].content == CellContent::Number(0) {
                self.board[y][x].content = CellContent::Mine;
                mines_placed += 1;
            }
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

    fn reveal(&mut self, x: usize, y: usize) {
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

    fn flag(&mut self, x: usize, y: usize) {
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

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        // self.cursor_x = (self.cursor_x as isize + dx).clamp(0, self.width as isize - 1) as usize;
        // self.cursor_y = (self.cursor_y as isize + dy).clamp(0, self.height as isize - 1) as usize;
        self.cursor_x = ((self.cursor_x as isize + dx).rem_euclid(self.width as isize)) as usize;
        self.cursor_y = ((self.cursor_y as isize + dy).rem_euclid(self.height as isize)) as usize;
    }
}

struct Tui {
    stdout: std::io::Stdout,
    game: Game,
}

impl Tui {
    pub fn new(game: Game) -> Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Tui { stdout, game })
    }

    /// Gets the character and color for a cell, but not its formatting or cursor highlight.
    fn get_cell_style(&self, x: usize, y: usize, show_all: bool) -> (char, Color) {
        let cell = &self.game.board[y][x];

        match cell.state {
            CellState::Covered if !show_all => (COVERED, Color::DarkGrey),
            CellState::Flagged if !show_all => (FLAG, Color::Red),
            _ => match cell.content {
                CellContent::Mine => (BOMB, Color::Magenta),
                CellContent::Explosion => (EXPLOSION, Color::Magenta),
                CellContent::Number(0) => (EMPTY, Color::White),
                CellContent::Number(1) => ('1', Color::Blue),
                CellContent::Number(2) => ('2', Color::DarkGreen),
                CellContent::Number(3) => ('3', Color::Red),
                CellContent::Number(4) => ('4', Color::DarkBlue),
                CellContent::Number(5) => ('5', Color::DarkRed),
                CellContent::Number(6) => ('6', Color::DarkCyan),
                CellContent::Number(7) => ('7', Color::Black),
                CellContent::Number(8) => ('8', Color::DarkGrey),
                CellContent::Number(n) => (
                    // unreachable - at most 8 neighbours
                    char::from_digit(n as u32, 10).unwrap_or('?'),
                    Color::Yellow,
                ),
            },
        }
    }

    fn display_help(&mut self) -> Result<()> {
        queue!(self.stdout, Clear(ClearType::All))?;

        let help_content = [
            ("MINESWEEPER - HELP", Color::Cyan),
            ("", Color::White),
            ("OBJECTIVE: Clear all cells without mines", Color::White),
            ("", Color::White),
            ("CONTROLS:", Color::Yellow),
            ("  ↑↓←→ / hjkl    Move cursor", Color::White),
            ("  R / Enter      Reveal cell", Color::White),
            ("  F / Space      Toggle flag", Color::White),
            ("  H / ?          This help", Color::White),
            ("  N              New game (when over)", Color::White),
            ("  Q / Esc        Quit", Color::White),
            ("", Color::White),
            ("SYMBOLS:", Color::Yellow),
            (
                &format!("  {COVERED:>3} Covered     {FLAG} Flagged     {EMPTY:<2}Empty"),
                Color::White,
            ),
            (
                &format!("  1-8 Mine count  {BOMB} Mine        {EXPLOSION} Explosion"),
                Color::White,
            ),
            ("", Color::White),
            (
                "TIP: Numbers show how many mines touch that cell",
                Color::DarkGrey,
            ),
            ("", Color::White),
            ("Press any key to continue...", Color::Cyan),
        ];
        for (i, (text, color)) in help_content.iter().enumerate() {
            queue!(
                self.stdout,
                cursor::MoveTo(2, i as u16 + 1),
                SetForegroundColor(*color),
                Print(text)
            )?;
        }

        queue!(self.stdout, ResetColor)?;
        self.stdout.flush()?;
        let _ = event::read()?;
        Ok(())
    }

    /// Redraws the entire screen using explicit cursor positioning for stability.
    fn display(&mut self) -> Result<()> {
        queue!(self.stdout, Clear(ClearType::All))?;

        // --- Draw static text ---
        let name = format!("MINESWEEPER ({}x{})", self.game.width, self.game.height);
        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print(name),
            cursor::MoveTo(0, 1),
            SetForegroundColor(Color::DarkGrey),
            Print("Controls: ←↑↓→ Move | R Reveal | F Flag | Q Quit | ? Help")
        )?;

        // --- Draw game status ---
        let flags_placed = self
            .game
            .board
            .iter()
            .flatten()
            .filter(|c| c.state == CellState::Flagged)
            .count();

        let elapsed_seconds = if let Some(duration) = self.game.final_time {
            duration.as_secs()
        } else if let Some(start) = self.game.start_time {
            start.elapsed().as_secs()
        } else {
            0
        };

        const M: &str = "Press 'n' for a new game.";
        let status = match self.game.game_state {
            GameState::Playing => {
                format!("Mines: {} | Flags: {}", self.game.num_mines, flags_placed)
            }
            GameState::Won => {
                format!("🎉 You Won! Time: {elapsed_seconds}s. {M}")
            }
            GameState::Lost => {
                format!("💥 Game Over! Time: {elapsed_seconds}s. {M}")
            }
        };

        queue!(
            self.stdout,
            cursor::MoveTo(0, 3),
            SetForegroundColor(Color::White),
            Print(status)
        )?;

        let show_all = self.game.game_state != GameState::Playing;

        // --- Draw board with explicit cursor positioning ---
        for y in 0..self.game.height {
            for x in 0..self.game.width {
                // Calculate the top-left corner of the cell on the screen
                let screen_x = x as u16 * CELL_WIDTH + BOARD_OFFSET_X;
                let screen_y = y as u16 + BOARD_OFFSET_Y;

                // Determine cell style
                let (char, fg_color) = self.get_cell_style(x, y, show_all);
                let is_cursor = x == self.game.cursor_x && y == self.game.cursor_y;
                let bg_color = if is_cursor && self.game.game_state == GameState::Playing {
                    CURSOR_BG_COLOR
                } else {
                    Color::Black // Use a default background color
                };

                // Format the 3-character wide cell content
                let display_string = format!(" {char}");

                // Queue all commands for drawing one cell
                queue!(
                    self.stdout,
                    cursor::MoveTo(screen_x, screen_y),
                    SetForegroundColor(fg_color),
                    SetBackgroundColor(bg_color),
                    Print("   "), // clear
                    cursor::MoveTo(screen_x, screen_y),
                    Print(display_string),
                )?;
            }
        }

        queue!(self.stdout, ResetColor)?; // Reset colors at the very end
        self.stdout.flush()
    }

    fn game_loop(&mut self) -> Result<()> {
        loop {
            self.display()?;

            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                let is_game_over = self.game.game_state != GameState::Playing;
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('?') => self.display_help()?,
                    KeyCode::Char('n') if is_game_over => {
                        self.game =
                            Game::new(self.game.width, self.game.height, self.game.num_mines);
                    }
                    _ if is_game_over => {} // Ignore other input if game over
                    KeyCode::Up | KeyCode::Char('k') => self.game.move_cursor(0, -1),
                    KeyCode::Down | KeyCode::Char('j') => self.game.move_cursor(0, 1),
                    KeyCode::Left | KeyCode::Char('h') => self.game.move_cursor(-1, 0),
                    KeyCode::Right | KeyCode::Char('l') => self.game.move_cursor(1, 0),
                    KeyCode::Char('r') | KeyCode::Enter => {
                        self.game.reveal(self.game.cursor_x, self.game.cursor_y)
                    }
                    KeyCode::Char('f') | KeyCode::Char(' ') => {
                        self.game.flag(self.game.cursor_x, self.game.cursor_y)
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.width * args.height <= args.num_mines {
        println!("Too many mines!");
        std::process::exit(0);
    }

    let game = Game::new(args.width, args.height, args.num_mines);
    let mut tui = Tui::new(game)?;

    tui.game_loop()
}
