use std::fmt::Display;
use std::io;
use std::time::{Duration, Instant};

// We use crossterm for handling raw input events (keyboard presses)
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
// Ratatui handles the actual drawing of widgets to the terminal
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

use conway_game_of_rust::grid::{CellState, Grid};

// Sets the speed of the simulation, will be mutable in the future.
const TIME_BETWEEN_GENERATIONS: u64 = 150;

fn main() -> io::Result<()> {
    // Initialize the terminal interface (enters raw mode, clears screen)
    let mut terminal = ratatui::init();
    // Run the application loop
    let app_result = App::default().run(&mut terminal);
    // Restore terminal to normal state (leaves raw mode) upon exit
    ratatui::restore();
    app_result
}

/// The main application state.
/// This struct holds the "Model" (Grid) and the "Controller" state (cursor, modes).
#[derive(Default)]
pub struct App {
    grid: Grid,
    cursor_pos: (usize, usize), // Current (row, col) of the user's cursor
    selection_anchor: Option<(usize, usize)>, // Where the user started their visual selection (if any)
    mode: Mode,                               // Current input mode (Normal, Visual, Running)
    exit: bool,                               // Flag to break the main loop
}

/// Represents the current state of the interface.
/// Inspired by Vim's modal editing:
/// - NORMAL: Move cursor, toggle single cells.
/// - VISUAL: Select multiple cells to toggle at once.
/// - RUNNING: The simulation is active and updating.
#[derive(PartialEq)]
enum Mode {
    RUNNING,
    NORMAL,
    VISUAL,
}

// Display trait allows us to easily print the mode into the title bar
impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mode_str = match self {
            Self::NORMAL => "[NORMAL]",
            Self::RUNNING => "[RUNNING]",
            Self::VISUAL => "[VISUAL]",
        };
        write!(f, "{mode_str}")
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::NORMAL
    }
}

impl App {
    /// The main event loop.
    /// This handles drawing, input polling, and updating the simulation state.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_millis(TIME_BETWEEN_GENERATIONS);
        let mut last_tick = Instant::now();

        while !self.exit {
            // 1. Render the current state
            terminal.draw(|frame| self.draw(frame))?;

            // 2. Calculate remaining time in this frame to maintain consistent speed
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // 3. Poll for user input (non-blocking wait based on timeout)
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    // Only handle press events, ignore release/repeat for cleaner input
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key);
                    }
                }
            }

            // 4. Update the simulation if the timer has elapsed and we are RUNNING
            if last_tick.elapsed() >= tick_rate {
                if self.mode == Mode::RUNNING {
                    self.grid.next_generation();
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    /// Helper to bridge the App struct with Ratatui's widget system
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Handles all keyboard inputs.
    /// This acts as the "Controller," modifying state based on key codes.
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let (row, col) = self.cursor_pos;

        match key_event.code {
            // --- GLOBAL KEYS (Always Work) ---
            KeyCode::Char('q') => self.exit(),
            // Enter acts as the Play/Pause toggle
            KeyCode::Enter => {
                if self.mode == Mode::RUNNING {
                    self.mode = Mode::NORMAL
                } else {
                    self.mode = Mode::RUNNING
                }
            }
            // Esc always returns to a safe "Normal" state
            KeyCode::Esc => {
                self.mode = Mode::NORMAL;
                self.selection_anchor = None;
            }

            // --- MODE SWITCHING ---
            // 'v' enters Visual Mode (unless simulation is running)
            KeyCode::Char('v') if self.mode != Mode::RUNNING => {
                self.mode = Mode::VISUAL;
                self.selection_anchor = Some((row, col));
            }

            // --- MOVEMENT (Works in NORMAL and VISUAL mode) ---
            // Supports both Vim keys (hjkl) and Arrow keys.
            // Guarded by `if self.mode != Mode::RUNNING` to prevent cursor interference during sim.
            KeyCode::Left | KeyCode::Char('h') if self.mode != Mode::RUNNING => {
                if col > 0 {
                    self.cursor_pos.1 -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if self.mode != Mode::RUNNING => {
                if row < self.grid.height - 1 {
                    self.cursor_pos.0 += 1
                }
            }
            KeyCode::Up | KeyCode::Char('k') if self.mode != Mode::RUNNING => {
                if row > 0 {
                    self.cursor_pos.0 -= 1
                }
            }
            KeyCode::Right | KeyCode::Char('l') if self.mode != Mode::RUNNING => {
                if col < self.grid.width - 1 {
                    self.cursor_pos.1 += 1;
                }
            }

            // --- ACTIONS ---
            // 'r' to reset (clear) the board
            KeyCode::Char('r') => {
                if self.mode != Mode::RUNNING {
                    self.grid.reset();
                }
            }
            // Spacebar behavior changes based on context
            KeyCode::Char(' ') => match self.mode {
                Mode::NORMAL => {
                    // Simple toggle of the cell under cursor
                    self.grid.toggle_cell(row, col);
                }
                Mode::VISUAL => {
                    // Bulk toggle: flip all cells in the selected rectangle
                    if let Some((anchor_r, anchor_c)) = self.selection_anchor {
                        let (min_r, max_r, min_c, max_c) =
                            get_row_and_col_span(row, col, anchor_r, anchor_c);

                        self.grid.multi_toggle_cells(min_r, max_r, min_c, max_c);
                    }

                    // Return to normal mode after action (standard Vim-like behavior)
                    self.mode = Mode::NORMAL;
                    self.selection_anchor = None;
                }
                Mode::RUNNING => {} // Do nothing while running
            },
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

/// Helper function to calculate the bounding box of a selection.
/// Takes two corners (cursor and anchor) and returns (min_row, max_row, min_col, max_col).
fn get_row_and_col_span(
    cursor_r: usize,
    cursor_c: usize,
    anchor_r: usize,
    anchor_c: usize,
) -> (usize, usize, usize, usize) {
    let min_r = cursor_r.min(anchor_r);
    let max_r = cursor_r.max(anchor_r);
    let min_c = cursor_c.min(anchor_c);
    let max_c = cursor_c.max(anchor_c);
    (min_r, max_r, min_c, max_c)
}

/// The main UI rendering logic.
/// Ratatui calls this to paint the `App` onto the `Frame`.
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Construct the title bar
        let title = Line::from(format!(" Conway's Game of Rust {}", self.mode).bold());

        // Dynamic help text at the bottom based on current mode
        let instructions = {
            match self.mode {
                Mode::NORMAL => Line::from(vec![
                    " Reset ".into(),
                    "<R>".blue().bold(),
                    " Selection Movement ".into(),
                    "hjkl / ← ↓ ↑ →".blue().bold(),
                    " Pause/Unpause Simulation ".into(),
                    "<Enter>".blue().bold(),
                    " Toggle Selected Cell(s) ".into(),
                    "<Space>".blue().bold(),
                    " Visual Mode ".into(),
                    "<V>".blue().bold(),
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ]),
                Mode::RUNNING => Line::from(vec![
                    " Pause/Unpause Simulation ".into(),
                    "<Enter>".blue().bold(),
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ]),
                Mode::VISUAL => Line::from(vec![
                    " Reset ".into(),
                    "<R>".blue().bold(),
                    " Selection Movement ".into(),
                    "hjkl / ← ↓ ↑ →".blue().bold(),
                    " Pause/Unpause Simulation ".into(),
                    "<Enter>".blue().bold(),
                    " Toggle Selected Cell(s) ".into(),
                    "<Space>".blue().bold(),
                    " Normal Mode ".into(),
                    "<Esc>".blue().bold(),
                    " Quit ".into(),
                    "<Q> ".blue().bold(),
                ]),
            }
        };

        // Create the border block
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let mut grid_lines = Vec::new();

        // --- Render the Grid ---
        for r in 0..self.grid.height {
            let mut row_spans = Vec::new();

            for c in 0..self.grid.width {
                // Determine the character symbol (Block for Alive, Dotted for Dead)
                let symbol = match self.grid.get(r, c) {
                    Some(CellState::Alive) => "██",
                    _ => "░░",
                };

                // Check if the current cell falls inside the visual selection box
                let is_in_selection = if self.mode == Mode::VISUAL {
                    if let Some((anchor_r, anchor_c)) = self.selection_anchor {
                        let (cursor_r, cursor_c) = self.cursor_pos;

                        let (min_r, max_r, min_c, max_c) =
                            get_row_and_col_span(cursor_r, cursor_c, anchor_r, anchor_c);

                        r >= min_r && r <= max_r && c >= min_c && c <= max_c
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Apply styling (Colors) based on state:
                // 1. Cursor position (White bg/Gray fg)
                // 2. Selection area (Blue theme)
                // 3. Normal cell (White fg)
                let style = if (r, c) == self.cursor_pos && self.mode != Mode::RUNNING {
                    Style::default().fg(Color::DarkGray).bg(Color::White)
                } else if is_in_selection {
                    match self.grid.get(r, c) {
                        Some(CellState::Alive) => {
                            Style::default().bg(Color::White).fg(Color::LightBlue)
                        }
                        _ => Style::default().bg(Color::LightBlue).fg(Color::White),
                    }
                } else {
                    Style::default().fg(Color::White)
                };

                row_spans.push(Span::styled(symbol, style));
            }

            grid_lines.push(Line::from(row_spans));
        }

        let grid_text = Text::from(grid_lines);

        // Render the text inside the block
        Paragraph::new(grid_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
