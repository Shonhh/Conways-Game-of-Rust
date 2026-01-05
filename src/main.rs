use std::fmt::Display;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

use conway_game_of_rust::grid::{CellState, Grid};

const TIME_BETWEEN_GENERATIONS: u64 = 150;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Default)]
pub struct App {
    grid: Grid,
    cursor_pos: (usize, usize),
    selection_anchor: Option<(usize, usize)>,
    mode: Mode,
    exit: bool,
}

#[derive(PartialEq)]
enum Mode {
    RUNNING,
    NORMAL,
    VISUAL,
}

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
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_millis(TIME_BETWEEN_GENERATIONS);
        let mut last_tick = Instant::now();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            // calc how much time left in this frame
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // poll for events
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key);
                    }
                }
            }

            // update game state if it's time
            if last_tick.elapsed() >= tick_rate {
                if self.mode == Mode::RUNNING {
                    self.grid.next_generation();
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    // implement numbered commands
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let (row, col) = self.cursor_pos;

        match key_event.code {
            // GLOBAL KEYS (Always Work)
            KeyCode::Char('q') => self.exit(),
            KeyCode::Enter => {
                if self.mode == Mode::RUNNING {
                    self.mode = Mode::NORMAL
                } else {
                    self.mode = Mode::RUNNING
                }
            }
            KeyCode::Esc => {
                self.mode = Mode::NORMAL;
                self.selection_anchor = None;
            }

            // MODE SWITCHING
            KeyCode::Char('v') if self.mode != Mode::RUNNING => {
                self.mode = Mode::VISUAL;
                self.selection_anchor = Some((row, col));
            }

            // MOVEMENT (Works in NORMAL and VISUAL mode)
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

            // ACTIONS
            KeyCode::Char('r') => {
                if self.mode != Mode::RUNNING {
                    self.grid.reset();
                }
            }
            KeyCode::Char(' ') => match self.mode {
                // toggle selected cell(s)
                Mode::NORMAL => {
                    self.grid.toggle_cell(row, col);
                }
                Mode::VISUAL => {
                    if let Some((anchor_r, anchor_c)) = self.selection_anchor {
                        let (min_r, max_r, min_c, max_c) =
                            get_row_and_col_span(row, col, anchor_r, anchor_c);

                        self.grid.multi_toggle_cells(min_r, max_r, min_c, max_c);
                    }

                    // Return to normal mode after event.
                    self.mode = Mode::NORMAL;
                    self.selection_anchor = None;
                }
                Mode::RUNNING => {}
            },
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(format!(" Conway's Game of Rust {}", self.mode).bold());
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

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let mut grid_lines = Vec::new();

        for r in 0..self.grid.height {
            let mut row_spans = Vec::new();

            for c in 0..self.grid.width {
                let symbol = match self.grid.get(r, c) {
                    Some(CellState::Alive) => "██",
                    _ => "░░",
                };

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

                // fix for toggled cells
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

        Paragraph::new(grid_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
