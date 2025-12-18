//! TUI implementation of the game.
//!
//! This module provides a full TUI implementation using the `crossterm` and `tui` crates.
//!
//! It exposes a function `run_tui` that starts the TUI game.

#[cfg(feature = "mine_tui")]
mod enabled {
    use std::error::Error;
    use std::io;
    use std::time::Duration;

    use crossterm::{
        event::{self, Event as CEvent, KeyCode, KeyEvent},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use tui::{
        Terminal,
        backend::CrosstermBackend,
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Span, Spans},
        widgets::{Block, Borders, Clear, Paragraph},
    };

    use crate::cells::{CellType, Status};
    use crate::game_play::Board;

    /// Full TUI implementation.
    pub fn run_tui(
        rows: usize,
        cols: usize,
        bomb_coords: Vec<(usize, usize)>,
    ) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let num_bombs = bomb_coords.len();
        let mut board = Board::from_bomb_coords((rows, cols), &bomb_coords);

        let mut cursor_r: usize = 0;
        let mut cursor_c: usize = 0;

        loop {
            let game_over = board.game_over;
            let win = board.check_win();
            let game_ended = game_over || win;

            terminal.draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
                    .split(size);

                let (rows, cols) = board.size;
                let mut lines = Vec::with_capacity(rows);
                for r in 0..rows {
                    let mut spans = Vec::with_capacity(cols * 2);
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

                        if r == cursor_r && c == cursor_c {
                            spans.push(Span::styled(
                                format!("{} ", ch),
                                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
                            ));
                        } else {
                            spans.push(Span::raw(format!("{} ", ch)));
                        }
                    }
                    lines.push(Spans::from(spans));
                }

                let board_block = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Mines (TUI)"));
                f.render_widget(board_block, chunks[0]);

                let info_text = if game_ended {
                    "Press r to restart • q to quit"
                } else {
                    "Controls: arrow keys/hjkl to move • Space/Enter to reveal • f to flag • r to restart • q to quit"
                };

                let info_lines = vec![
                    Spans::from(vec![Span::raw(info_text)]),
                    Spans::from(vec![Span::raw(format!(
                        "Cursor: ({},{})  •  Game over: {}  •  Win: {}",
                        cursor_r,
                        cursor_c,
                        board.game_over,
                        board.check_win()
                    ))]),
                ];
                let info_block = Paragraph::new(info_lines).block(Block::default().borders(Borders::ALL).title("Info"));
                f.render_widget(info_block, chunks[1]);

                if game_ended {
                    let area = centered_rect(60, 20, size);
                    f.render_widget(Clear, area);

                    let (title, color, msg) = if win {
                        ("YOU WON!", Color::Green, "Congratulations! You cleared the board.")
                    } else {
                        ("GAME OVER", Color::Red, "Boom! You hit a bomb.")
                    };

                    let block = Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .style(Style::default().fg(color));

                    let text = vec![
                        Spans::from(Span::styled(msg, Style::default().add_modifier(Modifier::BOLD))),
                        Spans::from(""),
                        Spans::from("Press 'r' to restart"),
                        Spans::from("Press 'q' to quit"),
                    ];

                    let paragraph = Paragraph::new(text)
                        .block(block)
                        .alignment(Alignment::Center);

                    f.render_widget(paragraph, area);
                }
            })?;

            if event::poll(Duration::from_millis(200))? {
                match event::read()? {
                    CEvent::Key(KeyEvent { code, .. }) => match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            break;
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            let new_bombs = Board::random_bomb_coords((rows, cols), num_bombs);
                            board = Board::from_bomb_coords((rows, cols), &new_bombs);
                            cursor_r = 0;
                            cursor_c = 0;
                        }
                        _ if !game_ended => match code {
                            KeyCode::Left | KeyCode::Char('h') => {
                                if cursor_c > 0 {
                                    cursor_c -= 1;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                if cursor_c + 1 < board.size.1 {
                                    cursor_c += 1;
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if cursor_r > 0 {
                                    cursor_r -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if cursor_r + 1 < board.size.0 {
                                    cursor_r += 1;
                                }
                            }
                            KeyCode::Char('f') | KeyCode::Char('F') => {
                                let _ = board.cycle_flag_rc(cursor_r, cursor_c);
                            }
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                let _ = board.reveal_rc(cursor_r, cursor_c);
                            }
                            _ => { /* ignore other keys */ }
                        },
                        _ => { /* ignore keys when game ended */ }
                    },
                    _ => { /* ignore mouse/resize events in this stub */ }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Helper function to center a rect to show the game over message
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }
}

/// Terminal User Interface (TUI) for the Minesweeper game, when feature is enabled
#[cfg(feature = "mine_tui")]
pub use enabled::run_tui;

/// Function to prevent errors when TUI is not enabled
#[cfg(not(feature = "mine_tui"))]
pub fn run_tui(
    _: usize,
    _: usize,
    _: Vec<(usize, usize)>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "TUI is not enabled.\nPlease enable the feature by running `cargo run --features mine_tui -- tui <args>`."
    );
    Ok(())
}
