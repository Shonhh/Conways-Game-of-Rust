use core::panic;
use std::fmt;

/// An enum that represents the state of an individual cell.
#[derive(Copy, Clone, PartialEq)]
pub enum CellState {
    Alive,
    Dead,
}

impl CellState {
    /// Helper to flip the state. Used for user interactions.
    fn toggle(&self) -> Self {
        match self {
            CellState::Alive => CellState::Dead,
            CellState::Dead => CellState::Alive,
        }
    }
}

/// A struct which holds the data for the grid.
///
/// IMPLEMENTATION NOTE:
/// Uses a single flattened `Vec<CellState>` instead of a `Vec<Vec<CellState>>`.
/// This improves CPU cache locality and performance, as the entire grid is contiguous
/// in memory. We calculate 2D indices manually using `row * width + col`.
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<CellState>,
}

impl Default for Grid {
    fn default() -> Self {
        // Default size suitable for most terminal windows
        Grid::new(128, 80)
    }
}

impl Grid {
    /// Initialize a new Grid of size `width` * `height`, with all values
    /// being `CellState::Dead`.
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![CellState::Dead; width * height];
        Grid {
            width,
            height,
            cells,
        }
    }

    /// Returns Some(CellState) if coordinates in bounds, None otherwise.
    pub fn get(&self, row: usize, col: usize) -> Option<&CellState> {
        let index = self.get_index_from_coords(row, col);
        self.cells.get(index)
    }

    /// Sets the given coordinate to `new_state`, doing nothing if
    /// coordinates are out of bounds.
    pub fn set(&mut self, row: usize, col: usize, new_state: CellState) {
        let index = self.get_index_from_coords(row, col);
        if let Some(cur_state) = self.cells.get_mut(index) {
            *cur_state = new_state;
        }
    }

    /// Flips a single cell at (row, col) from Alive->Dead or Dead->Alive.
    pub fn toggle_cell(&mut self, row: usize, col: usize) {
        if let Some(state) = self.get(row, col) {
            self.set(row, col, state.toggle())
        }
    }

    /// Flips a rectangular region of cells. Used by Visual Mode.
    pub fn multi_toggle_cells(&mut self, min_r: usize, max_r: usize, min_c: usize, max_c: usize) {
        for r in min_r..=max_r {
            for c in min_c..=max_c {
                self.toggle_cell(r, c)
            }
        }
    }

    /// Clears the board (sets all cells to Dead).
    pub fn reset(&mut self) {
        self.cells = vec![CellState::Dead; self.width * self.height];
    }

    // fn set_alive(&mut self, coords: &[(usize, usize)]) {
    //     for &(r, c) in coords {
    //         self.set(r, c, CellState::Alive);
    //     }
    // }

    // fn load_pattern(&mut self, pattern: &str, start_row: usize, start_col: usize) {
    //     for (row_offset, line) in pattern.trim().lines().enumerate() {
    //         for (col_offset, ch) in line.trim().chars().enumerate() {
    //             if ch == '#' {
    //                 self.set(start_row + row_offset, start_col + col_offset, CellState::Alive);
    //             }
    //         }
    //     }
    // }

    /// Helper to get the associated 1D index from a 2D `x` and `y` coordinate.
    fn get_index_from_coords(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }

    /// Calculate the next state of the grid.
    /// 1. Create a new vector buffer.
    /// 2. Calculate the state for every cell based on neighbors.
    /// 3. Swap the old vector with the new one.
    pub fn next_generation(&mut self) {
        let mut resulting_cells = Vec::with_capacity(self.width * self.height);
        for row in 0..self.height {
            for col in 0..self.width {
                resulting_cells.push(self.find_new_cell_state(row, col));
            }
        }

        self.cells = resulting_cells;
    }

    /// Applies the standard Game of Life rules to a single cell.
    fn find_new_cell_state(&self, r: usize, c: usize) -> CellState {
        let cur_state = match self.get(r, c) {
            Some(state) => state,
            None => panic!("coordinates out of bounds"),
        };

        let live_neighbors = self.count_live_neighbors(r, c);

        match cur_state {
            // Rule 1: Any live cell with 2 or 3 live neighbors lives.
            // Rule 2: Any live cell with <2 or >3 neighbors dies.
            CellState::Alive => match live_neighbors {
                2 | 3 => return CellState::Alive,
                _ => return CellState::Dead,
            },
            // Rule 3: Any dead cell with exactly 3 live neighbors becomes a live cell.
            CellState::Dead => {
                if live_neighbors == 3 {
                    return CellState::Alive;
                }
                CellState::Dead
            }
        }
    }

    /// Counts how many neighbors of a given cell are alive.
    /// Checks all 8 surrounding cells.
    fn count_live_neighbors(&self, row: usize, col: usize) -> usize {
        let row_i = row as isize;
        let col_i = col as isize;

        // Relative coordinates for the 8 neighbors
        const NEIGHBOR_OFFSETS: [(isize, isize); 8] = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        NEIGHBOR_OFFSETS
            .iter()
            .filter_map(|&(dr, dc)| {
                let neighbor_row_i = row_i + dr;
                let neighbor_col_i = col_i + dc;

                // 1. Boundary check: negative coordinates
                if neighbor_row_i < 0 || neighbor_col_i < 0 {
                    return None;
                }

                let (neighbor_row, neighbor_col) =
                    (neighbor_row_i as usize, neighbor_col_i as usize);

                // 2. Boundary check: exceeded width/height
                if neighbor_row >= self.height || neighbor_col >= self.width {
                    return None;
                }

                Some((neighbor_row, neighbor_col))
            })
            // 3. Check if the neighbor is actually alive
            .filter(|&(neighbor_row, neighbor_col)| {
                matches!(
                    self.get(neighbor_row, neighbor_col),
                    Some(&CellState::Alive)
                )
            })
            .count()
    }
}

/// Allows printing the grid to console/string.
/// Primarily used for debugging or simple text output, not the main TUI.
impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cells
            .chunks_exact(self.width)
            .map(|row| {
                row.iter()
                    .map(|state| match state {
                        CellState::Alive => '#',
                        CellState::Dead => '.',
                    })
                    .map(|ch| ch.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .try_for_each(|line| writeln!(f, "{}", line))?;

        Ok(())
    }
}
