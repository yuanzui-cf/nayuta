use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
pub struct Input {
    pub text: String,
    pub cursor: usize,
}
impl Input {
    pub fn new(text: String) -> Self {
        let cursor = text.len();
        Self { text, cursor }
    }
    pub fn insert(&mut self, text: &str) {
        if self.text.len() + text.len() <= 65536 {
            self.text.insert_str(self.cursor, text);
            self.cursor += text.len();
        }
    }
    fn previous(&self) -> usize {
        self.text[..self.cursor]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
    fn next(&self) -> usize {
        self.text[self.cursor..]
            .graphemes(true)
            .next()
            .map(|s| self.cursor + s.len())
            .unwrap_or(self.text.len())
    }
    pub fn key(&mut self, key: KeyEvent) {
        let control = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Left => self.cursor = self.previous(),
            KeyCode::Right => self.cursor = self.next(),
            KeyCode::Home | KeyCode::Char('a') if control || key.code == KeyCode::Home => {
                self.cursor = 0
            }
            KeyCode::End | KeyCode::Char('e') if control || key.code == KeyCode::End => {
                self.cursor = self.text.len()
            }
            KeyCode::Char('k') if control => {
                self.text.clear();
                self.cursor = 0;
            }
            KeyCode::Backspace => {
                let previous = self.previous();
                self.text.replace_range(previous..self.cursor, "");
                self.cursor = previous;
            }
            KeyCode::Delete => {
                let next = self.next();
                self.text.replace_range(self.cursor..next, "");
            }
            KeyCode::Char(c) if !control && !key.modifiers.contains(KeyModifiers::ALT) => {
                self.insert(&c.to_string())
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cursor_never_splits_cjk_or_combining_graphemes() {
        let mut input = Input::new("中e\u{301}😀".into());
        input.key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(input.text, "中e\u{301}");
        input.key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        input.insert("文");
        assert_eq!(input.text, "中文e\u{301}");
    }
}
