# minesweeper-rs

A classic game of Minesweeper [1,2] for the terminal, written in Rust and built with `crossterm`.

## Features

-   **Configurable Board:** Set the width, height, and number of mines.
-   **Vim Keybindings:** Navigate with `h`, `j`, `k`, `l` in addition to arrow keys.
-   **Safe First Click:** Never hit a mine on the first move.
-   **In-Game Help:** Press `?` anytime to see the controls.

## References

1. [Wikipedia](https://en.wikipedia.org/wiki/Minesweeper_(video_game)
2. [minesweeper.com](https://minesweepergame.com/)


## Installation

1. **Install Rust via [rustup.rs](https://rustup.rs/)**:
2. **Clone the repository:**
    ```bash
    git clone https://github.com/jesper-olsen/minesweeper-rs.git
    cd minesweeper-rs
    ```
3.  **Build the release binary:**
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/minesweeper-rs`.

## Usage

```bash
% cargo run --release -- --help

Usage: minesweeper-rs [OPTIONS]

Options:
      --width <WIDTH>          number of columns [default: 18]
      --height <HEIGHT>        number of rows [default: 10]
      --num-mines <NUM_MINES>  number of mines [default: 25]
  -h, --help                   Print help
  -V, --version                Print version
```

Run the executable directly to start a game with default settings:

```bash
./target/release/minesweeper-rs
```

| ![Game UI](Assets/screenshot.png) |
| --- |


## License

This project is licensed under the [MIT License](LICENSE).
