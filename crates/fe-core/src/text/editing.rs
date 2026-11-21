//! The text cursor and selection while text is typed on the canvas: inserting, deleting, moving by letter, word,
//! line, up and down, selecting. Pure logic: where the cursor can stand comes from the layout (`CaretPos`).

use super::layout::CaretPos;

/// The words being edited, the text cursor and the other end of the selection.
#[derive(Clone, Debug, PartialEq)]
pub struct TextCursor {
    pub chars: Vec<char>,
    pub caret: usize,
    /// The selection runs between `anchor` and `caret`; equal = no selection.
    pub anchor: usize,
    /// Moving up and down keeps the cursor near the same distance along the line.
    want_along: Option<f64>,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl TextCursor {
    /// Starts with the cursor at the end of `text`.
    pub fn new(text: &str) -> Self {
        let chars: Vec<char> = text.replace('\r', "").chars().collect();
        let end = chars.len();
        Self { chars, caret: end, anchor: end, want_along: None }
    }

    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn has_selection(&self) -> bool {
        self.caret != self.anchor
    }

    /// The selected range (start, end).
    pub fn range(&self) -> (usize, usize) {
        (self.caret.min(self.anchor), self.caret.max(self.anchor))
    }

    pub fn selected_text(&self) -> String {
        let (a, b) = self.range();
        self.chars[a..b].iter().collect()
    }

    fn go(&mut self, to: usize, select: bool) {
        self.caret = to.min(self.chars.len());
        if !select {
            self.anchor = self.caret;
        }
    }

    /// Puts the cursor at `index` (a mouse click); with `select` the selection grows to it (a drag).
    pub fn set_caret(&mut self, index: usize, select: bool) {
        self.want_along = None;
        self.go(index, select);
    }

    pub fn select_all(&mut self) {
        self.want_along = None;
        self.anchor = 0;
        self.caret = self.chars.len();
    }

    /// Selects the word around `index` (a double-click); on a space or a line break, just that stretch.
    pub fn select_word(&mut self, index: usize) {
        self.want_along = None;
        let n = self.chars.len();
        let at = index.min(n);
        let probe = if at < n { at } else { n.saturating_sub(1) };
        let Some(&c) = self.chars.get(probe) else { return };
        let same = |ch: char| if c == '\n' { ch == '\n' } else { is_word(ch) == is_word(c) && ch != '\n' };
        let (mut a, mut b) = (probe, probe + 1);
        while a > 0 && same(self.chars[a - 1]) {
            a -= 1;
        }
        while b < n && same(self.chars[b]) {
            b += 1;
        }
        self.anchor = a;
        self.caret = b;
    }

    /// Types text in place of the selection. Returns true when something changed.
    pub fn insert(&mut self, text: &str) -> bool {
        let new: Vec<char> = text.replace('\r', "").chars().filter(|c| *c == '\n' || !c.is_control()).collect();
        if new.is_empty() && !self.has_selection() {
            return false;
        }
        self.want_along = None;
        let (a, b) = self.range();
        self.chars.splice(a..b, new.iter().copied());
        self.caret = a + new.len();
        self.anchor = self.caret;
        true
    }

    /// Backspace: the selection, or the letter (with `word`: the word) before the cursor.
    pub fn backspace(&mut self, word: bool) -> bool {
        if self.has_selection() {
            return self.insert("");
        }
        if self.caret == 0 {
            return false;
        }
        let from = if word { self.word_start(self.caret) } else { self.caret - 1 };
        self.anchor = from;
        self.insert("")
    }

    /// Delete: the selection, or the letter (with `word`: the word) after the cursor.
    pub fn delete(&mut self, word: bool) -> bool {
        if self.has_selection() {
            return self.insert("");
        }
        if self.caret >= self.chars.len() {
            return false;
        }
        let to = if word { self.word_end(self.caret) } else { self.caret + 1 };
        self.anchor = to;
        self.insert("")
    }

    fn word_start(&self, from: usize) -> usize {
        let mut i = from;
        while i > 0 && !is_word(self.chars[i - 1]) && self.chars[i - 1] != '\n' {
            i -= 1;
        }
        while i > 0 && is_word(self.chars[i - 1]) {
            i -= 1;
        }
        i
    }

    fn word_end(&self, from: usize) -> usize {
        let n = self.chars.len();
        let mut i = from;
        while i < n && !is_word(self.chars[i]) && self.chars[i] != '\n' {
            i += 1;
        }
        while i < n && is_word(self.chars[i]) {
            i += 1;
        }
        i
    }

    /// Left arrow: a letter, or a word with `word`. With a selection and no `select` the cursor goes to its start.
    pub fn left(&mut self, select: bool, word: bool) {
        self.want_along = None;
        if self.has_selection() && !select {
            let (a, _) = self.range();
            self.go(a, false);
        } else if word {
            self.go(self.word_start(self.caret), select);
        } else {
            self.go(self.caret.saturating_sub(1), select);
        }
    }

    pub fn right(&mut self, select: bool, word: bool) {
        self.want_along = None;
        if self.has_selection() && !select {
            let (_, b) = self.range();
            self.go(b, false);
        } else if word {
            self.go(self.word_end(self.caret), select);
        } else {
            self.go(self.caret + 1, select);
        }
    }

    /// Home: the start of the line (with `whole`: of the text).
    pub fn home(&mut self, select: bool, whole: bool) {
        self.want_along = None;
        let mut i = self.caret;
        if whole {
            i = 0;
        } else {
            while i > 0 && self.chars[i - 1] != '\n' {
                i -= 1;
            }
        }
        self.go(i, select);
    }

    pub fn end(&mut self, select: bool, whole: bool) {
        self.want_along = None;
        let n = self.chars.len();
        let mut i = self.caret;
        if whole {
            i = n;
        } else {
            while i < n && self.chars[i] != '\n' {
                i += 1;
            }
        }
        self.go(i, select);
    }

    /// Up / down a line (`delta` = -1 / 1), staying near the same distance along the line.
    pub fn vertical(&mut self, delta: i32, select: bool, carets: &[CaretPos]) {
        let Some(here) = carets.get(self.caret) else { return };
        let want = *self.want_along.get_or_insert(here.along);
        let line = here.line as i32 + delta;
        let target = carets
            .iter()
            .filter(|c| c.line as i32 == line)
            .min_by(|a, b| (a.along - want).abs().total_cmp(&(b.along - want).abs()));
        let to = match target {
            Some(c) => c.index,
            None if delta < 0 => 0,
            None => self.chars.len(),
        };
        let keep = self.want_along;
        self.go(to, select);
        self.want_along = keep;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_deleting_and_replacing_the_selection() {
        let mut c = TextCursor::new("Hello");
        assert!(c.insert(" world"));
        assert_eq!("Hello world", c.text());
        c.home(false, false);
        c.right(true, true); // select "Hello"
        assert_eq!("Hello", c.selected_text());
        c.insert("Bye");
        assert_eq!("Bye world", c.text());
        assert!(c.backspace(true)); // the cursor is after "Bye": the word goes
        assert_eq!(" world", c.text());
        assert!(!c.backspace(false), "nothing before the start");
        assert!(c.delete(false));
        assert_eq!("world", c.text());
        assert!(c.insert(
            "a
b"
        ));
        assert_eq!(
            "a
bworld",
            c.text()
        );
    }

    #[test]
    fn moving_by_word_and_line_and_picking_a_word() {
        let mut c = TextCursor::new(
            "one two
three",
        );
        c.home(false, true);
        c.right(false, true);
        assert_eq!(3, c.caret);
        c.right(false, true);
        assert_eq!(7, c.caret);
        c.end(false, false);
        assert_eq!(7, c.caret, "end of the first line");
        c.home(false, false);
        assert_eq!(0, c.caret);
        c.select_word(9);
        assert_eq!("three", c.selected_text());
        c.left(false, false);
        assert_eq!((8, 8), c.range(), "the cursor goes to the start of the selection");
        c.select_all();
        assert_eq!(13, c.selected_text().chars().count());
    }

    #[test]
    fn up_and_down_stay_near_the_same_place_on_the_line() {
        let caret = |index, line, along| CaretPos { index, line, along, a: (0.0, 0.0), b: (0.0, 0.0) };
        // "abcd" and "ab": two lines.
        let carets: Vec<CaretPos> = (0..=4)
            .map(|i| caret(i, 0, i as f64 * 10.0))
            .chain((5..=7).map(|i| caret(i, 1, (i - 5) as f64 * 10.0)))
            .collect();
        let mut c = TextCursor::new(
            "abcd
ab",
        );
        c.set_caret(3, false);
        c.vertical(1, false, &carets);
        assert_eq!(7, c.caret, "the second line is shorter: its end");
        c.vertical(-1, false, &carets);
        assert_eq!(3, c.caret, "back where it was");
        c.vertical(-1, false, &carets);
        assert_eq!(0, c.caret, "no line above: the start");
    }
}
