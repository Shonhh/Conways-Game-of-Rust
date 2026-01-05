use core::panic;
use std::fmt;

/// An enum that represents the state of an individual cell.
#[derive(Copy, Clone, PartialEq)]
pub enum CellState {
    Alive,
    Dead,
}

impl CellState {
    fn toggle(&self) -> Self {
        match self {
            CellState::Alive => CellState::Dead,
            CellState::Dead => CellState::Alive,
        }
    }
}

/// A struct which holds the data for the folded vector of `CellState`s,
/// including its `width` and `height`.
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<CellState>,
}

impl Default for Grid {
    fn default() -> Self {
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

    pub fn toggle_cell(&mut self, row: usize, col: usize) {
        if let Some(state) = self.get(row, col) {
            self.set(row, col, state.toggle())
        }
    }

    pub fn multi_toggle_cells(&mut self, min_r: usize, max_r: usize, min_c: usize, max_c: usize) {
        for r in min_r..=max_r {
            for c in min_c..=max_c {
                self.toggle_cell(r, c)
            }
        }
    }

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

    /// Because `cells` is a flattened 2D `Vec<>`, use this function to get
    /// the associated index from an `x` and `y` coordinate.
    fn get_index_from_coords(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }

    /// Calculate the next state of the grid. Apply **Game of Life Rules** and append
    /// results to a new vector, then replace the `cells` field with the new vector.
    pub fn next_generation(&mut self) {
        let mut resulting_cells = Vec::with_capacity(self.width * self.height);
        for row in 0..self.height {
            for col in 0..self.width {
                resulting_cells.push(self.find_new_cell_state(row, col));
            }
        }

        self.cells = resulting_cells;
    }

    /// Returns what the state of a single cell will be in the next generation.
    fn find_new_cell_state(&self, r: usize, c: usize) -> CellState {
        let cur_state = match self.get(r, c) {
            Some(state) => state,
            None => panic!("coordinates out of bounds"),
        };

        let live_neighbors = self.count_live_neighbors(r, c);

        match cur_state {
            CellState::Alive => match live_neighbors {
                2 | 3 => return CellState::Alive,
                _ => return CellState::Dead,
            },
            CellState::Dead => {
                if live_neighbors == 3 {
                    return CellState::Alive;
                }
                CellState::Dead
            }
        }
    }

    /// Counts how many neighbors of a given cell are alive.
    fn count_live_neighbors(&self, row: usize, col: usize) -> usize {
        let row_i = row as isize;
        let col_i = col as isize;

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

                // skip negative coordinates
                if neighbor_row_i < 0 || neighbor_col_i < 0 {
                    return None;
                }

                let (neighbor_row, neighbor_col) =
                    (neighbor_row_i as usize, neighbor_col_i as usize);

                if neighbor_row >= self.height || neighbor_col >= self.width {
                    return None;
                }

                Some((neighbor_row, neighbor_col))
            })
            .filter(|&(neighbor_row, neighbor_col)| {
                matches!(
                    self.get(neighbor_row, neighbor_col),
                    Some(&CellState::Alive)
                )
            })
            .count()
    }
}

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
