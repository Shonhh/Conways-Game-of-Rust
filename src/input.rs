struct Input {
    cur_string: String,
}

impl Input {
    pub fn new() -> Self {
        Input {
            cur_string: String::new(),
        }
    }

    pub fn append_char(&mut self, c: char) {
        self.cur_string.push(c);

        if let Some(action) = c.to_digit(10) {
            return;
        }

        // Action is NOT a number, need to do something.
    }
}
