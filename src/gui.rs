//! GUI implementation of the game using Yew.
//!
//! This module exposes a function called `run_app` that starts the GUI application, listening to the URL: `http://localhost:8080

/// When the `mine_gui` is enabled we are going to want to run this code instead
#[cfg(feature = "mine_gui")]
mod enabled {
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::*;
    use web_sys::HtmlInputElement;
    use yew::prelude::*;

    use crate::cells::{CellType, Status};
    use crate::game_play::Board;

    /// The main Yew functional component for the Minesweeper GUI.
    ///
    /// This component manages the game state (the `Board`), user configuration for rows, columns, and number of bombs, and handles user interactions like revealing cells, toggling flags, and restarting the game.
    #[function_component(App)]
    pub fn app() -> Html {
        let initial_rows = 9;
        let initial_cols = 9;
        let initial_bombs = 10;

        let rows_state = use_state(|| initial_rows);
        let cols_state = use_state(|| initial_cols);
        let num_bombs_state = use_state(|| initial_bombs);

        let board_rc = use_state(|| {
            Rc::new(RefCell::new(Board::from_bomb_coords(
                (initial_rows, initial_cols),
                &Board::random_bomb_coords((initial_rows, initial_cols), initial_bombs),
            )))
        });
        let tick = use_state(|| 0usize);

        let create_new_board = {
            let board_rc = board_rc.clone();
            let rows_state = rows_state.clone();
            let cols_state = cols_state.clone();
            let num_bombs_state = num_bombs_state.clone();
            let tick = tick.clone();

            Callback::from(move |_| {
                let current_rows = *rows_state;
                let current_cols = *cols_state;
                let current_num_bombs = *num_bombs_state;

                if current_rows == 0
                    || current_cols == 0
                    || current_num_bombs > current_rows * current_cols
                {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message("Invalid board configuration: Rows and columns must be > 0. Number of bombs cannot exceed total cells.")
                        .unwrap();
                    return;
                }

                let mut b = board_rc.borrow_mut();
                *b = Board::from_bomb_coords(
                    (current_rows, current_cols),
                    &Board::random_bomb_coords((current_rows, current_cols), current_num_bombs),
                );
                tick.set(*tick + 1);
            })
        };

        let on_restart_game = create_new_board.clone();

        let handle_rows_change = {
            let rows_state = rows_state.clone();
            Callback::from(move |e: Event| {
                let input: HtmlInputElement = e.target_unchecked_into();
                if let Ok(val) = input.value().parse::<usize>() {
                    rows_state.set(val);
                }
            })
        };

        let handle_cols_change = {
            let cols_state = cols_state.clone();
            Callback::from(move |e: Event| {
                let input: HtmlInputElement = e.target_unchecked_into();
                if let Ok(val) = input.value().parse::<usize>() {
                    cols_state.set(val);
                }
            })
        };

        let handle_bombs_change = {
            let num_bombs_state = num_bombs_state.clone();
            Callback::from(move |e: Event| {
                let input: HtmlInputElement = e.target_unchecked_into();
                if let Ok(val) = input.value().parse::<usize>() {
                    num_bombs_state.set(val);
                }
            })
        };

        let board = board_rc.borrow();
        let (rows, cols) = board.size;
        let is_game_over_or_won = board.game_over || board.check_win();

        let rows_html: Html = (0..rows)
            .map(|r| {
                let cols_html: Html = (0..cols)
                    .map(|c| {
                        let cell_idx = r * cols + c;
                        let cell = &board.board[cell_idx];

                        let label = match cell.status {
                            Status::Hidden => ".".to_string(),
                            Status::Flag => "F".to_string(),
                            Status::Question => "?".to_string(),
                            Status::Revealed => match cell.cell_type {
                                CellType::Bomb => "B".to_string(),
                                CellType::Number(0) => " ".to_string(),
                                CellType::Number(n) => n.to_string(),
                            },
                        };

                        let onclick = if is_game_over_or_won {
                            Callback::from(|_| ())
                        } else {
                            let board_rc_clone = board_rc.clone();
                            let tick_clone = tick.clone();
                            Callback::from(move |_| {
                                let mut b_mut = board_rc_clone.borrow_mut();
                                let _ = b_mut.reveal_rc(r, c);
                                tick_clone.set(*tick_clone + 1);
                            })
                        };

                        let oncontextmenu = if is_game_over_or_won {
                            Callback::from(|e: MouseEvent| {
                                e.prevent_default();
                            })
                        } else {
                            let board_rc_clone = board_rc.clone();
                            let tick_clone = tick.clone();
                            Callback::from(move |e: MouseEvent| {
                                e.prevent_default();
                                let mut b_mut = board_rc_clone.borrow_mut();
                                let _ = b_mut.cycle_flag_rc(r, c);
                                tick_clone.set(*tick_clone + 1);
                            })
                        };

                        let button_style = format!(
                            "width: 32px; height: 32px; margin: 2px; padding: 0; display: inline-flex; align-items: center; justify-content: center; font-weight: bold; border: 1px solid #999; cursor: {}; background-color: {}; color: {};",
                            if is_game_over_or_won { "default" } else { "pointer" },
                            if cell.is_revealed() {
                                if cell.is_bomb() {
                                    "lightcoral"
                                } else if cell.number_value() == Some(0) {
                                    "lightgray"
                                } else {
                                    "white"
                                }
                            } else if cell.is_flagged() {
                                "lightblue"
                            } else if cell.is_question() {
                                "lightyellow"
                            } else {
                                "#ccc"
                            },
                            if cell.is_revealed() && cell.is_bomb() { "white" }
                            else if cell.is_revealed() && cell.number_value().unwrap_or(0) > 0 { "darkblue" }
                            else { "black" }
                        );

                        html! {
                            <button
                                {onclick}
                                {oncontextmenu}
                                disabled={is_game_over_or_won}
                                style={button_style}
                            >
                                { label }
                            </button>
                        }
                    })
                    .collect();

                html! {
                    <div style="display: flex; flex-direction: row;">{ cols_html }</div>
                }
            })
            .collect();

        let status_text = {
            if board.game_over {
                "Game Over! You hit a bomb.".to_string()
            } else if board.check_win() {
                "Congratulations! You won!".to_string()
            } else {
                format!(
                    "Board: {}x{} • Bombs: {}",
                    board.size.0, board.size.1, *num_bombs_state
                )
            }
        };

        html! {
            <div style="font-family: sans-serif; padding: 12px; max-width: fit-content; margin: auto; background-color: #f0f0f0; border-radius: 8px; box-shadow: 0 4px 8px rgba(0,0,0,0.1);">
                <h1 style="text-align: center; color: #333;">{ "Minesweeper" }</h1>

                <div style="margin-bottom: 20px; padding: 12px; border: 1px solid #ddd; background-color: #f9f9f9; border-radius: 4px;">
                    <h3 style="margin-top: 0; color: #555;">{ "Game Setup" }</h3>
                    <div style="display: flex; flex-wrap: wrap; gap: 15px; align-items: center;">
                        <div>
                            <label style="margin-right: 5px; font-weight: bold;">{ "Rows:" }</label>
                            <input type="number" min="1" value={rows_state.to_string()} onchange={handle_rows_change} style="width: 60px; padding: 5px; border: 1px solid #ccc; border-radius: 3px;" />
                        </div>
                        <div>
                            <label style="margin-right: 5px; font-weight: bold;">{ "Columns:" }</label>
                            <input type="number" min="1" value={cols_state.to_string()} onchange={handle_cols_change} style="width: 60px; padding: 5px; border: 1px solid #ccc; border-radius: 3px;" />
                        </div>
                        <div>
                            <label style="margin-right: 5px; font-weight: bold;">{ "Bombs:" }</label>
                            <input type="number" min="1" max={((*rows_state) * (*cols_state)).to_string()} value={num_bombs_state.to_string()} onchange={handle_bombs_change} style="width: 60px; padding: 5px; border: 1px solid #ccc; border-radius: 3px;" />
                        </div>
                        <button onclick={create_new_board} style="padding: 8px 16px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 1em; transition: background-color 0.2s;">{ "New Game" }</button>
                    </div>
                </div>

                <div style="border: 2px solid #555; padding: 5px; display: inline-block; background-color: #ddd; border-radius: 4px; box-shadow: inset 0 0 5px rgba(0,0,0,0.2);">
                    { rows_html }
                </div>

                <div style="margin-top: 20px; text-align: center; display: flex; flex-direction: column; align-items: center;">
                    <em style="font-size: 1.2em; font-weight: bold; margin-bottom: 15px; color: #333;">{ status_text }</em>
                    <button onclick={on_restart_game} style="padding: 10px 20px; font-size: 1.1em; background-color: #4CAF50; color: white; border: none; border-radius: 5px; cursor: pointer; transition: background-color 0.2s;">{ "Restart Game" }</button>
                </div>

                <div style="margin-top: 30px; color: #666; font-size: 0.9em; text-align: center; border-top: 1px dashed #ccc; padding-top: 15px;">
                    { "Left click = reveal • Right click = cycle flag/question (Hidden → Flag → Question → Hidden)" }
                </div>
            </div>
        }
    }

    /// Bootstrap function called when the WASM module is instantiated.
    /// 
    /// This mounts the Yew `App` component onto the document body.
    /// 
    /// Use `wasm-bindgen` to export the start function so it runs automatically.
    #[wasm_bindgen(start)]
    pub fn run_app() {
        yew::Renderer::<App>::new().render();
    }
}

/// This provides the actual `run_app` function publicly when the `mine_gui` feature is enabled.
#[cfg(feature = "mine_gui")]
pub use enabled::run_app;

/// This is a placeholder function for the code to compile even if the `mine_gui` feature is not enabled.
#[cfg(not(feature = "mine_gui"))]
pub fn run_app() {
    // GUI feature is disabled; no-op at runtime.
}
