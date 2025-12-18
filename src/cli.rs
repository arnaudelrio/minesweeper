//! Cli module of the minesweeper game.
//!
//! This module exposes [`run()`] which starts a minimal interactive REPL that accepts commands to reveal and flag cells. It's intentionally small and avoids any external dependencies so it can be used as a starting point for both tests and integration with other frontends.

use std::io::{self, Write};

use crate::cells::{CellType, Status};
use crate::game_play::Board;

/// Simple CLI frontend for the Minesweeper board logic.
///
/// This function starts a minimal interactive REPL that accepts commands to reveal and flag cells. It's intentionally small and avoids any external dependencies so it can be used as a starting point for both tests and integration with other frontends.
///
/// Possible commands:
/// - `reveal x y`: Reveal the cell at position (x, y).
/// - `flag x y`: Flag the cell at position (x, y).
/// - `help`: Display a list of available commands.
/// - `quit`: Quit the game.
pub fn run(rows: usize, cols: usize, bomb_coords: Vec<(usize, usize)>) {
    let mut board = Board::from_bomb_coords((rows, cols), &bomb_coords);

    println!("Minesweeper CLI");
    println!("Board size: {} rows x {} cols", rows, cols);
    println!("Commands:");
    println!("  r <row> <col>  - reveal at (row,col)");
    println!("  f <row> <col>  - cycle flag at (row,col) (Hidden -> Flag -> Question -> Hidden)");
    println!("  h              - help");
    println!("  q              - quit");
    println!("Rows and cols are 0-based indices (0..rows-1, 0..cols-1).");
    println!();

    loop {
        print_board_console(&board);
        if board.game_over {
            println!("Game over. A bomb was revealed.");
            break;
        }
        if board.check_win() {
            println!("You won! All safe cells revealed.");
            break;
        }

        print!("> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input - exiting.");
            break;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts[0].to_lowercase().as_str() {
            "q" | "quit" | "exit" => {
                println!("Bye.");
                break;
            }
            "h" | "help" => {
                println!("Commands:");
                println!("  r <row> <col>  - reveal at (row,col)");
                println!(
                    "  f <row> <col>  - cycle flag at (row,col) (Hidden -> Flag -> Question -> Hidden)"
                );
                println!("  h              - help");
                println!("  q              - quit");
                println!("Rows and cols are 0-based indices (0..rows-1, 0..cols-1).");
                println!();
            }
            "r" | "reveal" => {
                if parts.len() < 3 {
                    println!("Usage: r <row> <col>");
                    continue;
                }
                let rr = match parts[1].parse::<usize>() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Invalid row: {}", parts[1]);
                        continue;
                    }
                };
                let cc = match parts[2].parse::<usize>() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Invalid col: {}", parts[2]);
                        continue;
                    }
                };
                match board.reveal_rc(rr, cc) {
                    Ok(n) => {
                        println!("Revealed ({},{}) => {}", rr, cc, n);
                    }
                    Err(e) if e == "FLAGGED" => {
                        println!("Cell ({},{}) is flagged. Unflag before revealing.", rr, cc);
                    }
                    Err(e) if e == "OUT_OF_BOUNDS" => {
                        println!("Coordinates out of bounds.");
                    }
                    Err(e) if e == "BOMB" => {
                        println!("Boom! You revealed a bomb at ({},{})", rr, cc);
                    }
                    Err(e) => {
                        println!("Reveal failed: {}", e);
                    }
                }
            }
            "f" | "flag" | "mark" => {
                if parts.len() < 3 {
                    println!("Usage: f <row> <col>");
                    continue;
                }
                let rr = match parts[1].parse::<usize>() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Invalid row: {}", parts[1]);
                        continue;
                    }
                };
                let cc = match parts[2].parse::<usize>() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Invalid col: {}", parts[2]);
                        continue;
                    }
                };
                match board.cycle_flag_rc(rr, cc) {
                    Ok(()) => {
                        println!("Cycled mark at ({},{})", rr, cc);
                    }
                    Err(e) if e == "OUT_OF_BOUNDS" => {
                        println!("Coordinates out of bounds.");
                    }
                    Err(e) if e == "REVEALED" => {
                        println!("Cell is already revealed.");
                    }
                    Err(e) => {
                        println!("Failed to change mark: {}", e);
                    }
                }
            }
            other => {
                println!("Unknown command: {}. Use r/f/q.", other);
            }
        }
    }
}

/// Render the board to a multi-line string and print it.
///
/// It has the column number and row number at the top and sides for the players to be able to identify the position of the cells withe ease
///
/// This representation is in the cli, and therefore consists of a simple ASCII view:
///  - Hidden cells: '.'
///  - Flagged cells: 'F'
///  - Question: '?'
///  - Revealed bomb: 'B'
///  - Revealed number: ' ' for 0, digit for 1..8 according to the number of surrounding bombs
fn print_board_console(board: &Board) {
    let (rows, cols) = board.size;
    let row_num_chars = (rows - 1).to_string().len();
    let col_num_chars = (cols - 1).to_string().len();
    println!();
    println!(
        "{:>row_num_chars$} |{}",
        "",
        (0..cols)
            .into_iter()
            .map(|x| format!(" {:0>col_num_chars$}", x))
            .collect::<String>()
    );
    println!(
        "{:->row_num_chars$}--{}",
        "-",
        (0..cols)
            .into_iter()
            .map(|_| format!("-{:-<col_num_chars$}", '-'))
            .collect::<String>()
    );
    for r in 0..rows {
        let mut line = String::with_capacity(cols * 2);
        for c in 0..cols {
            let idx = r * cols + c;
            let cell = &board.board[idx];
            let ch = match cell.status {
                Status::Hidden => '.',
                Status::Flag => 'F',
                Status::Question => '?',
                Status::Revealed => match cell.cell_type {
                    CellType::Bomb => 'B',
                    CellType::Number(0) => ' ',
                    CellType::Number(n) if (1..=9).contains(&n) => {
                        std::char::from_digit(n as u32, 10).unwrap_or('#')
                    }
                    CellType::Number(_) => '#',
                },
            };
            if c < cols {
                line.push_str(format!("{:<col_num_chars$}", " ").as_str());
            }
            line.push(ch);
        }
        println!("{:0>row_num_chars$} |{}", r, line);
    }
    println!();
}
