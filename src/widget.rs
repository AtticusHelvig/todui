use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use std::collections::HashMap;

/// Input Field Widget
struct InputField {
    lines: Vec<String>,
    count: usize,
    width: u16,
    height: u16,
}

impl InputField {
    pub fn new(input: String, size: (u16, u16)) -> Self {
        Self {
            lines: str_to_lines(input.as_str(), (size.0, size.1)),
            count: input.len(),
            width: size.0,
            height: size.1,
        }
    }

    pub fn set_input(&mut self, input: String) {
        self.lines = str_to_lines(input.as_str(), (self.width, self.height));
        self.cursor_cache = None;
    }

    pub fn get_cursor_at(&mut self, index: usize) -> (u16, u16) {
        let mut index = usize::min(index, self.count);
        let mut y = 0;

        while index > 0 {
            let line_len = self
                .lines
                .get(y)
                .expect("Valid lines should exist for all index <= count.")
                .len();
            if (index as i32 - line_len as i32) < 0 {
                return (index as u16, y as u16);
            }
            index -= line_len;
            y += 1;
        }
        panic!("Should always return early from loop...");
    }
}

impl Widget for &mut InputField {
    fn render(self, area: Rect, buf: &mut Buffer) {}
}

/// Converts a &str to a Vec<Span>
/// ONLY WORKS FOR ASCII STRINGS
fn str_to_lines(string: &str, size: (u16, u16)) -> Vec<String> {
    let width = size.0 as usize;
    let height = size.1 as usize;
    let mut result = Vec::new();

    for raw_line in string.lines() {
        let tokens = tokenize_ascii(raw_line);
        let mut line_start: Option<usize> = None;
        let mut line_end = 0;
        let mut current_len = 0;

        for &(start, end) in &tokens {
            let token_len = end - start;

            // If we encounter a token that is longer than a line
            if let Some(ls) = line_start {
                // start by flushing the line (unless it is empty)
                result.push(raw_line[ls..line_end].to_string());
                if result.len() >= height {
                    return result;
                }
                // Then break it up
                let mut pos = start;
                while pos < end {
                    let chunk_end = usize::min(pos + width, end);
                    result.push(raw_line[pos..chunk_end].to_string());
                    if result.len() >= height {
                        return result;
                    }
                    pos = chunk_end;
                }
                line_start = None;
                line_end = 0;
                continue;
            }
            // Check if the token fits on the line
            if current_len + token_len > width {
                // Flush the line if it doesn't
                if let Some(ls) = line_start {
                    result.push(raw_line[ls..line_end].to_string());
                    if result.len() >= height {
                        return result;
                    }
                }
                // Start new line with this token
                line_start = Some(start);
                line_end = end;
                current_len = token_len;
            } else {
                // Add to the current line
                if line_start.is_none() {
                    line_start = Some(start);
                }
                line_end = end;
                current_len += token_len;
            }
        }
        // Last the leftovers
        if let Some(ls) = line_start {
            result.push(raw_line[ls..line_end].to_string());
        }
    }
    result
}

/// Returns indexes to 'tokens' which are sequences of whitespace or words
/// ONLY WORKS ON ASCII
fn tokenize_ascii(input: &str) -> Vec<(usize, usize)> {
    let mut tokens = Vec::new();
    let mut start = 0;
    // Determine whether we start in a whitespace or word
    let mut in_whitespace = input
        .chars()
        .next()
        .map(|c| c.is_whitespace())
        .unwrap_or(false);

    for (i, c) in input.char_indices() {
        if !c.is_ascii() {
            panic!("Attempted to tokenize a non-ascii character.");
        }
        // End a token if we are in a whitespace and find a word
        // or are in a word and find a whitespace
        if c.is_whitespace() != in_whitespace {
            tokens.push((start, i));
            start = i;
            in_whitespace = c.is_whitespace();
        }
    }
    // Don't forget the leftovers
    if start < input.len() {
        tokens.push((start, input.len()));
    }
    tokens
}
