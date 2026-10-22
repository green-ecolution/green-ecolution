use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputOutcome {
    Editing,
    Submit(String),
    Cancel,
}

/// A single-line editor; `cursor` is a byte offset on a char boundary.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineInput {
    text: String,
    cursor: usize,
}

impl LineInput {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            cursor: text.len(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor_column(&self) -> u16 {
        u16::try_from(self.text[..self.cursor].chars().count()).unwrap_or(u16::MAX)
    }

    pub fn handle(&mut self, key: KeyEvent) -> InputOutcome {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Enter => return InputOutcome::Submit(self.text.clone()),
            KeyCode::Esc => return InputOutcome::Cancel,
            KeyCode::Char('u') if ctrl => {
                self.text.drain(..self.cursor);
                self.cursor = 0;
            }
            KeyCode::Char('w') if ctrl => {
                let start = self.previous_word_start();
                self.text.drain(start..self.cursor);
                self.cursor = start;
            }
            KeyCode::Char(c) if !ctrl => {
                self.text.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            KeyCode::Backspace => {
                if let Some(previous) = self.previous_boundary() {
                    self.text.drain(previous..self.cursor);
                    self.cursor = previous;
                }
            }
            KeyCode::Delete => {
                if let Some(next) = self.next_boundary() {
                    self.text.drain(self.cursor..next);
                }
            }
            KeyCode::Left => {
                if let Some(previous) = self.previous_boundary() {
                    self.cursor = previous;
                }
            }
            KeyCode::Right => {
                if let Some(next) = self.next_boundary() {
                    self.cursor = next;
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.text.len(),
            _ => {}
        }
        InputOutcome::Editing
    }

    fn previous_boundary(&self) -> Option<usize> {
        self.text[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(index, _)| index)
    }

    fn next_boundary(&self) -> Option<usize> {
        self.text[self.cursor..]
            .chars()
            .next()
            .map(|c| self.cursor + c.len_utf8())
    }

    fn previous_word_start(&self) -> usize {
        let before = self.text[..self.cursor].trim_end();
        before
            .char_indices()
            .rev()
            .find(|(_, c)| c.is_whitespace())
            .map_or(0, |(index, c)| index + c.len_utf8())
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    fn press(input: &mut LineInput, code: KeyCode) -> InputOutcome {
        input.handle(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn ctrl(input: &mut LineInput, c: char) -> InputOutcome {
        input.handle(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
    }

    fn typed(text: &str) -> LineInput {
        let mut input = LineInput::new("");
        for c in text.chars() {
            press(&mut input, KeyCode::Char(c));
        }
        input
    }

    #[test]
    fn edits_at_the_cursor() {
        let mut input = typed("lvel");
        press(&mut input, KeyCode::Home);
        press(&mut input, KeyCode::Right);
        press(&mut input, KeyCode::Char('e'));
        assert_eq!(input.text(), "level");
        assert_eq!(input.cursor_column(), 2);

        press(&mut input, KeyCode::End);
        press(&mut input, KeyCode::Backspace);
        press(&mut input, KeyCode::Home);
        press(&mut input, KeyCode::Delete);
        assert_eq!(input.text(), "eve");
    }

    #[test]
    fn handles_multibyte_characters() {
        let mut input = typed("grün");
        press(&mut input, KeyCode::Left);
        press(&mut input, KeyCode::Left);
        assert_eq!(input.cursor_column(), 2);
        press(&mut input, KeyCode::Delete);
        assert_eq!(input.text(), "grn");
    }

    #[test]
    fn deletes_words_and_lines_like_a_shell() {
        let mut input = typed("level>=warn unit:db  ");
        ctrl(&mut input, 'w');
        assert_eq!(input.text(), "level>=warn ");
        ctrl(&mut input, 'u');
        assert_eq!(input.text(), "");
    }

    #[test]
    fn enter_submits_and_escape_cancels() {
        let mut input = LineInput::new("unit:db");
        assert_eq!(input.cursor_column(), 7);
        assert_eq!(
            press(&mut input, KeyCode::Enter),
            InputOutcome::Submit("unit:db".into())
        );
        assert_eq!(press(&mut input, KeyCode::Esc), InputOutcome::Cancel);
        assert_eq!(press(&mut input, KeyCode::Char('x')), InputOutcome::Editing);
    }
}
