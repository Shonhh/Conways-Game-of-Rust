use core::panic;
use std::fmt;

/// An enum that represents the state of an individual cell.
#[derive(Debug, Copy, Clone, PartialEq)]
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
/// Boundaries are fixed: cells outside the grid are always dead.
pub struct Grid {
    width: usize,
    height: usize,
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
        assert!(width > 0 && height > 0, "grid dimensions must be positive");
        let cell_count = width.checked_mul(height).expect("grid dimensions overflow");
        let cells = vec![CellState::Dead; cell_count];
        Grid {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn live_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|&&cell| cell == CellState::Alive)
            .count()
    }

    /// Stable FNV-1a checksum over dimensions (u64 little endian), then
    /// row-major cells (dead = 0, alive = 1). This is not cryptographic.
    pub fn checksum(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        for byte in (self.width as u64)
            .to_le_bytes()
            .into_iter()
            .chain((self.height as u64).to_le_bytes())
            .chain(
                self.cells
                    .iter()
                    .map(|cell| u8::from(*cell == CellState::Alive)),
            )
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// Returns Some(CellState) if coordinates in bounds, None otherwise.
    pub fn get(&self, row: usize, col: usize) -> Option<&CellState> {
        self.get_index_from_coords(row, col)
            .and_then(|index| self.cells.get(index))
    }

    /// Sets the given coordinate to `new_state`, doing nothing if
    /// coordinates are out of bounds.
    pub fn set(&mut self, row: usize, col: usize, new_state: CellState) {
        if let Some(index) = self.get_index_from_coords(row, col) {
            self.cells[index] = new_state;
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
        for r in min_r..=max_r.min(self.height - 1) {
            for c in min_c..=max_c.min(self.width - 1) {
                self.toggle_cell(r, c)
            }
        }
    }

    /// Clears the board (sets all cells to Dead).
    pub fn reset(&mut self) {
        self.cells = vec![CellState::Dead; self.width * self.height];
    }

    /// Validate both coordinates before flattening, so an invalid column
    /// cannot alias a valid cell in the following row.
    fn get_index_from_coords(&self, row: usize, col: usize) -> Option<usize> {
        if row < self.height && col < self.width {
            Some(row * self.width + col)
        } else {
            None
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_with(width: usize, height: usize, live: &[(usize, usize)]) -> Grid {
        let mut grid = Grid::new(width, height);
        for &(row, col) in live {
            grid.set(row, col, CellState::Alive);
        }
        grid
    }

    fn live_cells(grid: &Grid) -> Vec<(usize, usize)> {
        (0..grid.height())
            .flat_map(|row| {
                (0..grid.width()).filter_map(move |col| {
                    (grid.get(row, col) == Some(&CellState::Alive)).then_some((row, col))
                })
            })
            .collect()
    }

    #[test]
    fn block_is_stable() {
        let cells = [(2, 2), (2, 3), (3, 2), (3, 3)];
        let mut grid = grid_with(7, 6, &cells);
        for _ in 0..10 {
            grid.next_generation();
            assert_eq!(live_cells(&grid), cells);
        }
    }

    #[test]
    fn blinker_has_period_two() {
        let horizontal = [(2, 1), (2, 2), (2, 3)];
        let mut grid = grid_with(5, 5, &horizontal);
        grid.next_generation();
        assert_eq!(live_cells(&grid), [(1, 2), (2, 2), (3, 2)]);
        grid.next_generation();
        assert_eq!(live_cells(&grid), horizontal);
    }

    #[test]
    fn glider_translates_after_four_generations() {
        let mut grid = grid_with(10, 9, &[(2, 3), (3, 4), (4, 2), (4, 3), (4, 4)]);
        for _ in 0..4 {
            grid.next_generation();
        }
        assert_eq!(live_cells(&grid), [(3, 4), (4, 5), (5, 3), (5, 4), (5, 5)]);
    }

    #[test]
    fn invalid_coordinates_do_not_alias_other_rows_or_overflow() {
        let mut grid = grid_with(4, 3, &[(1, 0)]);
        let before = grid.checksum();
        for (row, col) in [(0, 4), (1, 4), (3, 0), (usize::MAX, 0), (0, usize::MAX)] {
            assert_eq!(grid.get(row, col), None);
            grid.set(row, col, CellState::Alive);
            grid.toggle_cell(row, col);
        }
        assert_eq!(grid.checksum(), before);
        assert_eq!(grid.get(1, 0), Some(&CellState::Alive));
    }

    #[test]
    fn corner_blocks_are_stable_with_dead_exterior() {
        for (top, left) in [(0, 0), (0, 3), (2, 0), (2, 3)] {
            let cells = [
                (top, left),
                (top, left + 1),
                (top + 1, left),
                (top + 1, left + 1),
            ];
            let mut grid = grid_with(5, 4, &cells);
            grid.next_generation();
            assert_eq!(live_cells(&grid), cells);
        }
    }

    #[test]
    fn opposite_edges_do_not_wrap() {
        let mut grid = grid_with(5, 5, &[(0, 4), (4, 0), (4, 4)]);
        grid.next_generation();
        assert_eq!(grid.live_count(), 0);
    }

    #[test]
    fn top_edge_blinker_loses_exterior_birth() {
        let mut grid = grid_with(5, 5, &[(0, 1), (0, 2), (0, 3)]);
        grid.next_generation();
        assert_eq!(live_cells(&grid), [(0, 2), (1, 2)]);
    }

    #[test]
    fn tiny_and_empty_grids_update() {
        for (width, height) in [(1, 1), (1, 4), (4, 1)] {
            let mut grid = grid_with(width, height, &[(0, 0)]);
            grid.next_generation();
            assert_eq!(grid.live_count(), 0);
            grid.next_generation();
            assert_eq!(grid.live_count(), 0);
        }
    }

    #[test]
    fn every_local_rule_combination_is_correct() {
        let offsets = [
            (1, 1),
            (1, 2),
            (1, 3),
            (2, 1),
            (2, 3),
            (3, 1),
            (3, 2),
            (3, 3),
        ];
        for mask in 0_u16..256 {
            for center_alive in [false, true] {
                let mut grid = Grid::new(5, 5);
                for (bit, &(row, col)) in offsets.iter().enumerate() {
                    if mask & (1 << bit) != 0 {
                        grid.set(row, col, CellState::Alive);
                    }
                }
                if center_alive {
                    grid.set(2, 2, CellState::Alive);
                }
                grid.next_generation();
                let count = mask.count_ones();
                let expected = count == 3 || (center_alive && count == 2);
                assert_eq!(
                    grid.get(2, 2) == Some(&CellState::Alive),
                    expected,
                    "mask={mask}, center_alive={center_alive}"
                );
            }
        }
    }

    #[test]
    fn rectangle_toggle_clips_and_reset_preserves_dimensions() {
        let mut grid = Grid::new(4, 3);
        grid.multi_toggle_cells(1, usize::MAX, 2, usize::MAX);
        assert_eq!(live_cells(&grid), [(1, 2), (1, 3), (2, 2), (2, 3)]);
        grid.reset();
        assert_eq!((grid.width(), grid.height()), (4, 3));
        assert_eq!(grid.live_count(), 0);
    }

    #[test]
    fn checksum_includes_shape_and_cell_positions() {
        let first = grid_with(4, 3, &[(1, 0)]);
        assert_ne!(first.checksum(), grid_with(4, 3, &[(0, 1)]).checksum());
        assert_ne!(Grid::new(4, 3).checksum(), Grid::new(3, 4).checksum());
        assert_eq!(first.checksum(), grid_with(4, 3, &[(1, 0)]).checksum());
    }

    #[test]
    #[should_panic(expected = "grid dimensions must be positive")]
    fn zero_width_is_rejected() {
        Grid::new(0, 3);
    }

    #[test]
    #[should_panic(expected = "grid dimensions must be positive")]
    fn zero_height_is_rejected() {
        Grid::new(3, 0);
    }

    #[test]
    #[should_panic(expected = "grid dimensions overflow")]
    fn overflowing_dimensions_are_rejected() {
        Grid::new(usize::MAX, 2);
    }
}
