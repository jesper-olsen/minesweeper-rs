use crate::game::{CellContent, CellState, Game, GameState};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Result, Write};

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
const BOARD_OFFSET_X: u16 = 2;
const BOARD_OFFSET_Y: u16 = 5;

pub struct Tui {
    stdout: std::io::Stdout,
    cursor_x: usize,
    cursor_y: usize,
    game: Game,
    show_bomb_probability: bool,
}

impl Tui {
    pub fn new(game: Game, show_bomb_probability: bool) -> Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
        let cursor_x = game.width / 2;
        let cursor_y = game.height / 2;
        Ok(Tui {
            stdout,
            game,
            cursor_x,
            cursor_y,
            show_bomb_probability,
        })
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        // edges are hard - don't move cursor over
        // self.cursor_x = (self.cursor_x as isize + dx).clamp(0, self.width as isize - 1) as usize;
        // self.cursor_y = (self.cursor_y as isize + dy).clamp(0, self.height as isize - 1) as usize;
        // wrap around when cursor moves over edge
        self.cursor_x =
            ((self.cursor_x as isize + dx).rem_euclid(self.game.width as isize)) as usize;
        self.cursor_y =
            ((self.cursor_y as isize + dy).rem_euclid(self.game.height as isize)) as usize;
    }

    /// Gets the character and color for a cell, but not its formatting or cursor highlight.
    fn get_cell_style(&self, x: usize, y: usize, show_all: bool) -> (char, Color) {
        let cell = self.game.get_cell(x, y);

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

        // Wait for a real key press (ignore releases and repeats)
        loop {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Redraws the entire screen using explicit cursor positioning for stability.
    fn display(&mut self) -> Result<()> {
        //queue!(self.stdout, Clear(ClearType::All))?;
        // --- Draw static text ---
        let name = format!(
            "{BOMB} MINESWEEPER{BOMB}  ({}x{}, {} mines)",
            self.game.width, self.game.height, self.game.num_mines
        );
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
        let elapsed_seconds = if let Some(duration) = self.game.final_time {
            duration.as_secs()
        } else if let Some(start) = self.game.start_time {
            start.elapsed().as_secs()
        } else {
            0
        };

        const M: &str = "Press 'n' for a new game.           "; // extra space: ensure line is cleared
        let status = match self.game.state {
            GameState::Playing => {
                let flags = self.game.count(CellState::Flagged);
                let covered = self.game.count(CellState::Covered);

                let prob_display = if self.show_bomb_probability {
                    let prob = if covered + flags == self.game.width * self.game.height {
                        self.game.num_mines as f64 / (covered + flags) as f64
                    } else {
                        self.game.get_bomb_prob(self.cursor_x, self.cursor_y)
                    };
                    format!(
                        " | Mine @ ({},{}): {prob:4.2}",
                        self.cursor_x, self.cursor_y
                    )
                } else {
                    String::new()
                };

                format!(
                    "Mines: {} | Flags: {flags} | Covered: {covered}{prob_display}              ",
                    self.game.num_mines
                )
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

        let show_all = self.game.state != GameState::Playing;

        // --- Draw board with explicit cursor positioning ---
        for y in 0..self.game.height {
            for x in 0..self.game.width {
                // Calculate the top-left corner of the cell on the screen
                let screen_x = x as u16 * CELL_WIDTH + BOARD_OFFSET_X;
                let screen_y = y as u16 + BOARD_OFFSET_Y;

                // Determine cell style
                let (char, fg_color) = self.get_cell_style(x, y, show_all);
                let is_cursor = x == self.cursor_x && y == self.cursor_y;
                let bg_color = if is_cursor && self.game.state == GameState::Playing {
                    CURSOR_BG_COLOR
                } else {
                    Color::Black
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

    pub fn game_loop(&mut self) -> Result<()> {
        loop {
            self.display()?;

            if let Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) = event::read()?
            {
                let is_game_over = self.game.state != GameState::Playing;
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('?') => self.display_help()?,
                    KeyCode::Char('n') if is_game_over => {
                        self.game = Game::new(
                            self.game.width,
                            self.game.height,
                            self.game.num_mines,
                            self.game.first_click_policy,
                        );
                    }
                    _ if is_game_over => {} // Ignore other input if game over
                    KeyCode::Up | KeyCode::Char('k') => self.move_cursor(0, -1),
                    KeyCode::Down | KeyCode::Char('j') => self.move_cursor(0, 1),
                    KeyCode::Left | KeyCode::Char('h') => self.move_cursor(-1, 0),
                    KeyCode::Right | KeyCode::Char('l') => self.move_cursor(1, 0),
                    KeyCode::Char('r') | KeyCode::Enter => {
                        self.game.reveal(self.cursor_x, self.cursor_y)
                    }
                    KeyCode::Char('f') | KeyCode::Char(' ') => {
                        self.game.flag(self.cursor_x, self.cursor_y)
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
