//! Main game logic and board implementation.

use std::collections::VecDeque;

use rand::Rng;

use crate::cells::{Cell, CellType};

/// Game Board structure
///
/// Every new game uses an instance of this struct.
///
/// This is the main structure used in the game and it basically consists of an array of Cells.
pub struct Board {
    /// Board size: rows, cols
    pub size: (usize, usize),
    /// Board cells. It should be of length `size.0` * `size.1`
    pub board: Vec<Cell>,
    /// Game Over status
    pub game_over: bool,
}

/// This is the implementation of the Board struct.
///
/// It provides methods for creating a new game board and generating random bomb coordinates, as well as revealing cells.
impl Board {
    /// Create an empty board (all cells are Number(0) and Hidden).
    ///
    /// Use [`Board::from_bomb_coords()`] to create a board that contains bombs.
    pub fn new_game(size: (usize, usize)) -> Board {
        let (rows, cols) = size;
        let mut board = Vec::with_capacity(rows * cols);
        for _ in 0..rows * cols {
            board.push(Cell::new_number(0));
        }
        Board {
            size,
            board,
            game_over: false,
        }
    }

    /// Generate random coordinates for bombs.
    ///
    /// The function uses a random number generator to select unique coordinates within the given range.
    pub fn random_bomb_coords(size: (usize, usize), num_bombs: usize) -> Vec<(usize, usize)> {
        let mut rng = rand::rng();
        let mut bomb_coords = Vec::with_capacity(num_bombs);
        while bomb_coords.len() < num_bombs {
            let r = rng.random_range(0..size.0);
            let c = rng.random_range(0..size.1);
            if !bomb_coords.contains(&(r, c)) {
                bomb_coords.push((r, c));
            }
        }
        bomb_coords
    }

    /// Create a board initialized with bombs at the given coordinates.
    ///
    /// `bomb_coords` is a slice of `(row, col)` tuples. Coordinates outside the board are ignored. After placing bombs the adjacent counts for number cells are computed.
    pub fn from_bomb_coords(size: (usize, usize), bomb_coords: &[(usize, usize)]) -> Board {
        let mut b = Board::new_game(size);

        for &(r, c) in bomb_coords {
            if let Some(idx) = b.rc_to_index_checked(r, c) {
                b.board[idx] = Cell::new_bomb();
            }
        }

        let (rows, cols) = size;
        for r in 0..rows {
            for c in 0..cols {
                if let Some(idx) = b.rc_to_index_checked(r, c) {
                    if b.board[idx].is_bomb() {
                        continue;
                    }
                    let mut count = 0;
                    for n_idx in b.neighbors_indices(idx) {
                        if b.board[n_idx].is_bomb() {
                            count += 1;
                        }
                    }
                    b.board[idx] = Cell::new_number(count);
                }
            }
        }

        b
    }

    /// Convert (row, col) to linear index if in bounds.
    fn rc_to_index_checked(&self, r: usize, c: usize) -> Option<usize> {
        let (rows, cols) = self.size;
        if r < rows && c < cols {
            Some(r * cols + c)
        } else {
            None
        }
    }

    /// Convert linear index to (row, col).
    fn index_to_rc(&self, idx: usize) -> (usize, usize) {
        let cols = self.size.1;
        (idx / cols, idx % cols)
    }

    /// Return neighbor indices (8-way) for a given index.
    fn neighbors_indices(&self, idx: usize) -> Vec<usize> {
        let (r, c) = self.index_to_rc(idx);
        let (rows, cols) = self.size;
        let mut out = Vec::with_capacity(8);

        let r_min = if r == 0 { 0 } else { r - 1 };
        let c_min = if c == 0 { 0 } else { c - 1 };
        let r_max = if r + 1 >= rows { rows - 1 } else { r + 1 };
        let c_max = if c + 1 >= cols { cols - 1 } else { c + 1 };

        for rr in r_min..=r_max {
            for cc in c_min..=c_max {
                if rr == r && cc == c {
                    continue;
                }
                out.push(rr * cols + cc);
            }
        }

        out
    }

    /// Count the number of bombs revealed around the current cell
    pub fn count_flagged_neighbors(&mut self, idx: usize) -> usize {
        self.neighbors_indices(idx)
            .iter()
            .filter(|&&n_idx| self.board[n_idx].is_flagged())
            .count()
    }

    /// Reveal the cell at `idx`.
    ///
    /// Returns:
    /// * `Ok(number)` if a number cell was revealed (the number value).
    /// * `Err("BOMB")` if a bomb was revealed (and sets `game_over`).
    /// * `Err("OUT_OF_BOUNDS")` for invalid indices.
    ///
    /// If the revealed cell is a zero (0) number, a flood fill reveal (BFS) is performed revealing contiguous zeroes and their border number cells.
    pub fn reveal(&mut self, idx: usize) -> Result<i32, String> {
        if idx >= self.board.len() {
            return Err(String::from("OUT_OF_BOUNDS"));
        }

        if self.board[idx].is_revealed() {
            if self.board[idx].number_value().unwrap_or(0)
                == self.count_flagged_neighbors(idx) as i32
            {
                self.neighbors_indices(idx).iter().for_each(|&n_idx| {
                    if !self.board[n_idx].is_revealed() {
                        self.reveal(n_idx).unwrap_or_default();
                    }
                });
            }
            return match self.board[idx].number_value() {
                Some(n) => Ok(n),
                None => Err(String::from("BOMB")),
            };
        }

        if self.board[idx].is_flagged() {
            return Err(String::from("FLAGGED"));
        }

        match self.board[idx].reveal() {
            CellType::Bomb => {
                self.game_over = true;
                Err(String::from("BOMB"))
            }
            CellType::Number(n) => {
                if n == 0 {
                    self.flood_reveal(idx);
                }
                Ok(n)
            }
        }
    }

    /// Reveal neighbors recursively starting from a zero cell using BFS.
    ///
    /// This reveals neighboring zeroes and stops at number > 0 cells (they are revealed).
    fn flood_reveal(&mut self, start_idx: usize) {
        let mut q = VecDeque::new();
        let mut visited = vec![false; self.board.len()];
        q.push_back(start_idx);
        visited[start_idx] = true;

        while let Some(idx) = q.pop_front() {
            if self.board[idx].is_flagged() {
                continue;
            }

            let ctype = self.board[idx].reveal();

            match ctype {
                CellType::Bomb => {
                    continue;
                }
                CellType::Number(n) => {
                    if n == 0 {
                        for n_idx in self.neighbors_indices(idx) {
                            if !visited[n_idx]
                                && !self.board[n_idx].is_revealed()
                                && !self.board[n_idx].is_flagged()
                            {
                                visited[n_idx] = true;
                                q.push_back(n_idx);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Cycle a flag state for the given cell index using the Hidden->Flag->Question->Hidden cycle.
    ///
    /// Returns
    /// * `Ok(())` on success
    /// * `Err("OUT_OF_BOUNDS")` if index invalid
    /// * `Err("REVEALED")` if trying to change the status of an already revealed cell.
    pub fn cycle_flag(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.board.len() {
            return Err(String::from("OUT_OF_BOUNDS"));
        }

        if self.board[idx].is_revealed() {
            return Err(String::from("REVEALED"));
        }

        self.board[idx].cycle_status();
        Ok(())
    }

    /// Check if the player has won: All non-bomb cells are revealed.
    pub fn check_win(&self) -> bool {
        for cell in &self.board {
            if !cell.is_bomb() && !cell.is_revealed() {
                return false;
            }
        }
        true
    }

    /// Reveal by (row, col)
    ///
    /// This method is used because the board is stored as a 1-dimensional vector
    pub fn reveal_rc(&mut self, r: usize, c: usize) -> Result<i32, String> {
        if let Some(idx) = self.rc_to_index_checked(r, c) {
            self.reveal(idx)
        } else {
            Err(String::from("OUT_OF_BOUNDS"))
        }
    }

    /// Cycle flag by (row, col)
    ///
    /// This method is used because the board is stored as a 1-dimensional vector
    pub fn cycle_flag_rc(&mut self, r: usize, c: usize) -> Result<(), String> {
        if let Some(idx) = self.rc_to_index_checked(r, c) {
            self.cycle_flag(idx)
        } else {
            Err(String::from("OUT_OF_BOUNDS"))
        }
    }
}
