use crossterm::event::{KeyCode, KeyEvent};

const MAX_COUNT: usize = 10_000;

pub struct Input {
    keycodes: Vec<KeyEvent>,
    running_number: usize,
}

impl Input {
    pub fn new() -> Self {
        Input {
            keycodes: Vec::new(),
            running_number: 0,
        }
    }

    pub fn append_keycode(&mut self, k: KeyEvent) -> bool {
        if let KeyCode::Char(c) = k.code {
            if c.is_ascii_digit() {
                let digit = c.to_digit(10).unwrap() as usize;
                self.keycodes.push(k);
                self.running_number = self
                    .running_number
                    .saturating_mul(10)
                    .saturating_add(digit)
                    .min(MAX_COUNT);

                return false; // no action needed
            }
        }

        // action required, notify caller
        true
    }

    pub fn clear_buffer(&mut self) {
        self.keycodes.clear();
        self.running_number = 0;
    }

    pub fn running_number(&self) -> usize {
        if self.running_number == 0 {
            1
        } else {
            self.running_number
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Input::new()
    }
}
