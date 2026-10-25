use std::fmt::Write;

use crate::key_path::PATH_SEPARATOR;

const INDENT: &str = "    ";

// Writes keys given in tree order as nested KDL nodes, one line per key.
#[derive(Default)]
pub(crate) struct NestedWriter {
    text: String,
    open: Vec<String>,
}

impl NestedWriter {
    pub fn key(&mut self, path: &str, value: &str, comment: &str) {
        let segments: Vec<&str> = path.split(PATH_SEPARATOR).collect();
        let (leaf, parents) = segments.split_last().expect("a key path has at least one segment");
        let shared = self
            .open
            .iter()
            .zip(parents)
            .take_while(|(open, parent)| open == parent)
            .count();
        self.close_to(shared);

        for parent in &parents[shared..] {
            let _ = writeln!(self.text, "{}{parent} {{", INDENT.repeat(self.open.len()));
            self.open.push(parent.to_string());
        }

        let _ = writeln!(
            self.text,
            "{}{leaf} {value} // {comment}",
            INDENT.repeat(self.open.len())
        );
    }

    pub fn finish(mut self) -> String {
        self.close_to(0);

        self.text.trim_end().to_string()
    }

    fn close_to(&mut self, depth: usize) {
        while self.open.len() > depth {
            self.open.pop();
            let _ = writeln!(self.text, "{}}}", INDENT.repeat(self.open.len()));
        }
    }
}
