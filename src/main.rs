//! # Minesweeper game
//!
//! This is my implementation of the classic minesweeper game.
//!
//! This particular program can run as a _cli_, as a _tui_ or as a _gui_.

/// Function to recieve the args from the cli.
use std::env::{self, args};

mod cells;
mod cli;
mod game_play;
mod gui;
mod tui;

use cli::run;
use gui::run_app;
use tui::run_tui;

use crate::game_play::Board;

/// Main entrypoint of the program
///
/// This main function only acts as a way to select the execution mode. To run each mode use the following commands:
/// * **cli**: `cargo run -- cli`
/// * **tui**: `cargo run --features mine_tui -- tui`
/// * **gui**: `trunk serve`
fn main() {
    for arg in args() {
        match arg.as_str() {
            "cli" => start_cli(),
            "tui" => start_tui(),
            "gui" => start_gui(),
            _ => continue,
        }
    }
}

/// Handles the command line interface arguments.
///
/// Cli args:
/// - rows: Number of rows in the board.
/// - cols: Number of columns in the board.
/// - bomb_coords: Coordinates of the bombs.
///
/// If no bomb coordinates are provided, the function will generate a random board with a default number of bombs. If no row or column coordinates are provided, the function will generate a random board with a default number of rows and columns, which is `9`.
fn cli_args() -> (usize, usize, Vec<(usize, usize)>) {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 4 {
        let r = args[2].parse::<usize>().unwrap_or(9);
        let c = args[3].parse::<usize>().unwrap_or(9);
        if r <= 0 || c <= 0 {
            panic!("Board size invalid");
        }
        let bomb_args = if args.get(4).is_none() {
            vec![String::from(format!("{}", r * c / 10))]
        } else {
            args.clone().split_off(4)
        };
        let coords: Vec<(usize, usize)> = if !bomb_args[0].contains(",") {
            let num = bomb_args[0].parse::<usize>().unwrap_or(r * c / 10);
            if num > r * c {
                panic!("Invalid or too many bombs");
            }
            Board::random_bomb_coords((r, c), num)
        } else {
            let mut coords = Vec::new();
            for token in bomb_args {
                if let Some((rs, cs)) = token.split_once(',') {
                    if let (Ok(rr), Ok(cc)) = (rs.parse::<usize>(), cs.parse::<usize>()) {
                        coords.push((rr, cc));
                    }
                }
            }
            coords
        };
        (r, c, coords)
    } else {
        (9, 9, Board::random_bomb_coords((9, 9), 10))
    }
}

/// Starts the command line interface
fn start_cli() {
    let (rows, cols, bomb_coords) = cli_args();
    run(rows, cols, bomb_coords);
}

/// Starts the terminal user interface
fn start_tui() {
    let (rows, cols, bomb_coords) = cli_args();
    let _ = run_tui(rows, cols, bomb_coords);
}

/// Starts the graphical user interface
fn start_gui() {
    run_app();
}
