//! Cell types and utilities for the Mine Sweeper game.
//!
//! This module defines the [`CellType`], [`Status`] and [`Cell`] types along with convenience methods used by the game logic.

/// The kind of content a cell holds.
///
/// It can either be a [`CellType::Number`] indicating how many adjacent bombs it has, or a [`CellType::Bomb`] indicating a bomb cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    /// A numbered cell indicating how many adjacent bombs exist.
    Number(i32),
    /// A bomb cell.
    Bomb,
}

impl CellType {
    /// Returns [`true`] if this cell type is a [`CellType::Bomb`].
    pub fn is_bomb(&self) -> bool {
        matches!(self, CellType::Bomb)
    }

    /// If this is a [`CellType::Number`], returns its value, otherwise [`None`].
    pub fn number(&self) -> Option<i32> {
        match *self {
            CellType::Number(v) => Some(v),
            CellType::Bomb => None,
        }
    }
}

/// The visible/status state of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Hidden (not revealed and no flag/question).
    Hidden,
    /// Marked as question.
    Question,
    /// Marked as flagged (suspected [`CellType::Bomb`]).
    Flag,
    /// Cell has been revealed.
    Revealed,
}

impl Status {
    /// Cycle the status in the common Minesweeper order used for right-click:
    ///
    /// `Hidden -> Flag -> Question -> Hidden`
    pub fn cycle(&self) -> Status {
        match *self {
            Status::Hidden => Status::Flag,
            Status::Flag => Status::Question,
            Status::Question => Status::Hidden,
            Status::Revealed => Status::Revealed,
        }
    }
}

/// A single board cell.
#[derive(Clone, Copy)]
pub struct Cell {
    pub cell_type: CellType,
    pub status: Status,
}

impl Cell {
    /// Convenience constructor for a hidden numbered cell.
    pub fn new_number(n: i32) -> Cell {
        Cell {
            cell_type: CellType::Number(n),
            status: Status::Hidden,
        }
    }

    /// Convenience constructor for a hidden bomb cell.
    pub fn new_bomb() -> Cell {
        Cell {
            cell_type: CellType::Bomb,
            status: Status::Hidden,
        }
    }

    /// Toggle status in the `Hidden -> Flag -> Question -> Hidden` cycle.
    ///
    /// Revealed cells remain revealed.
    pub fn cycle_status(&mut self) {
        self.status = self.status.cycle();
    }

    /// Reveal the cell. The status will be set to [`Status::Revealed`].
    ///
    /// Returns the underlying [`CellType`].
    pub fn reveal(&mut self) -> CellType {
        self.status = Status::Revealed;
        self.cell_type
    }

    /// Return true if the cell is [`Status::Flag`].
    pub fn is_flagged(&self) -> bool {
        self.status == Status::Flag
    }

    /// Return true if the cell is [`Status::Question`].
    pub fn is_question(&self) -> bool {
        self.status == Status::Question
    }

    /// Return true if the cell is [`Status::Revealed`].
    pub fn is_revealed(&self) -> bool {
        self.status == Status::Revealed
    }

    /// Return true if the underlying type is a [`CellType::Bomb`].
    pub fn is_bomb(&self) -> bool {
        self.cell_type.is_bomb()
    }

    /// If this is a number cell, return its value.
    pub fn number_value(&self) -> Option<i32> {
        self.cell_type.number()
    }
}

/*impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cell")
            .field("cell_type", &self.cell_type)
            .field("status", &self.status)
            .finish()
    }
}
*/
