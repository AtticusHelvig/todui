use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::Widget;

/// Input Field Widget
#[derive(Default, Debug)]
pub struct InputField {
    input: String,
    wrapping: Wrap,
}

/// Represents different kinds of text wrapping
/// None: No wrapping at all, words flow off the line
/// Character: Characters wrap to the next line as necessary
/// Word: Words that don't fit on the current line go to the next
#[derive(Default, Debug)]
pub enum Wrap {
    #[default]
    None,
    Character,
    Word,
}

impl InputField {
    pub fn new(input: String, wrapping: Wrap) -> Self {
        Self { input, wrapping }
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    pub fn set_wrapping(&mut self, wrapping: Wrap) {
        self.wrapping = wrapping;
    }

    pub fn get_cursor_at(&self, area: Rect, index: usize) -> (u16, u16) {
        if self.input.len() == 0 {
            return (area.x, area.y);
        }

        let mut index = usize::min(index, self.input.len() - 1);
        let mut y = 0;
        let lines = self.lines(area);

        for line in lines {
            if index >= line.len() {
                index -= line.len();
                y += 1;
                continue;
            }
            return (area.x + index as u16, area.y + y as u16);
        }
        (area.x + area.width - 1, area.y + area.height - 1)
    }

    pub fn lines(&self, area: Rect) -> Vec<String> {
        match self.wrapping {
            Wrap::None => self.input.lines().map(str::to_string).collect(),
            Wrap::Character => todo!(),
            Wrap::Word => wrap_words(&self.input, (area.width, area.height)),
        }
    }
}

impl Widget for &InputField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut area = area;
        for line in self.lines(area) {
            Span::raw(line).render(area, buf);
            area.y += 1;
        }
    }
}

/// Converts a &str to a Vec<String> where each String is a line
/// Enforces word wrapping
/// ONLY WORKS FOR ASCII STRINGS
fn wrap_words(string: &str, size: (u16, u16)) -> Vec<String> {
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
            let last_line = result.len() == height - 1;
            let fits_on_line = current_len + token_len <= width;

            // Don't wrap the last line
            if last_line && !fits_on_line {
                let ls = match line_start {
                    Some(val) => val,
                    None => start,
                };
                let end = usize::min(ls + width, end);
                result.push(raw_line[ls..end].to_string());
                return result;
            }

            // If we encounter a token that is longer than a line
            if token_len > width {
                // start by flushing the line (unless it is empty)
                if let Some(ls) = line_start {
                    result.push(raw_line[ls..line_end].to_string());
                    if result.len() >= height {
                        return result;
                    }
                }
                // Then break it up
                let mut pos = start;
                while pos < end {
                    let chunk_end = usize::min(pos + width, end);
                    // If it flows off the line, start a new line
                    if pos + width <= end {
                        result.push(raw_line[pos..chunk_end].to_string());
                        line_start = None;
                        line_end = 0;
                    } else {
                        line_start = Some(pos);
                        line_end = end;
                        current_len = end - pos;
                    }
                    if result.len() >= height {
                        return result;
                    }
                    pos = chunk_end;
                }
                continue;
            }
            // Check if the token fits on the line
            if !fits_on_line {
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
