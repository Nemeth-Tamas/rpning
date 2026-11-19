//! Bounded history and literal substring search, separate from calculator undo.

use crate::{config::state_path, persistence};

#[derive(Debug, Clone, Default)]
pub struct History {
    entries: Vec<String>,
}

impl History {
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
    pub fn record(&mut self, line: &str) {
        if line.is_empty()
            || line.contains(['\n', '\r'])
            || line.len() > 4096
            || self.entries.last().is_some_and(|last| last == line)
        {
            return;
        }
        self.entries.push(line.to_owned());
        while self.entries.len() > 1000
            || self
                .entries
                .iter()
                .map(|line| line.len() + 1)
                .sum::<usize>()
                > 900_000
        {
            self.entries.remove(0);
        }
    }
    pub fn search<'a>(&'a self, needle: &'a str) -> impl Iterator<Item = (usize, &'a str)> + 'a {
        self.entries
            .iter()
            .enumerate()
            .filter(move |(_, line)| line.contains(needle))
            .map(|(i, line)| (i + 1, line.as_str()))
    }
    pub fn load() -> Result<Self, String> {
        let path = state_path("history.txt");
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = persistence::read_text(&path).map_err(|error| error.to_string())?;
        let mut lines = text.lines();
        if lines.next() != Some("RPNING-HISTORY-1") {
            return Err("unsupported history file version".to_owned());
        }
        let mut history = Self::default();
        for line in lines {
            history.record(line);
        }
        Ok(history)
    }
    pub fn save(&self) -> Result<(), String> {
        let text = format!("RPNING-HISTORY-1\n{}\n", self.entries.join("\n"));
        persistence::write_text(&state_path("history.txt"), &text)
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn history_is_bounded_deduplicated_and_searchable() {
        let mut history = History::default();
        for index in 0..1005 {
            history.record(&format!("{index} dup"));
        }
        history.record("1004 dup");
        assert_eq!(history.entries().len(), 1000);
        assert_eq!(
            history.search("1004").collect::<Vec<_>>(),
            [(1000, "1004 dup")]
        );
    }
}
