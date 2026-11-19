//! Testable calculator view model and bounded, responsive terminal layout.
use crate::{
    Calculator,
    config::Config,
    history::History,
    keymap::Keymap,
    keypad::{Key, Keypad, Outcome},
    programming, storage, workflow,
};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Input,
    Stack,
    History,
    Result,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Normal,
    Accent,
    Result,
    Input,
    Error,
    Muted,
}
pub struct Row {
    pub text: String,
    pub tone: Tone,
}
#[derive(Debug, Clone)]
pub struct Entry {
    pub source: String,
    pub result: String,
    pub failed: bool,
}

pub struct Tui {
    pub pad: Keypad,
    pub mapping: Keymap,
    pub focus: Focus,
    pub command_mode: bool,
    pub command: String,
    pub cursor: usize,
    pub history: History,
    pub journal: VecDeque<Entry>,
    pub message: String,
    pub failed: bool,
    pub stack_offset: usize,
    pub history_offset: usize,
    pub value_offset: usize,
    pub show_history: bool,
}

impl Tui {
    pub fn new(calculator: Calculator, config: Config) -> Self {
        Self {
            pad: Keypad::from_calculator(calculator),
            mapping: Keymap::new(config),
            focus: Focus::Input,
            command_mode: false,
            command: String::new(),
            cursor: 0,
            history: History::default(),
            journal: VecDeque::new(),
            message: "Ready - numeric input; F5 opens commands".to_owned(),
            failed: false,
            stack_offset: 0,
            history_offset: 0,
            value_offset: 0,
            show_history: false,
        }
    }
    pub fn result(&self) -> String {
        let calc = self.pad.calculator();
        calc.top()
            .map(|value| {
                self.mapping
                    .config
                    .display
                    .format(value, calc.settings().rounding)
            })
            .unwrap_or_else(|| "(empty)".to_owned())
    }
    fn record(&mut self, source: String, result: Result<String, String>) {
        let (text, failed) = match result {
            Ok(text) => (text, false),
            Err(error) => (error, true),
        };
        self.message = text.clone();
        self.failed = failed;
        self.journal.push_front(Entry {
            source,
            result: text,
            failed,
        });
        self.journal.truncate(256);
        self.history_offset = 0;
        self.value_offset = 0;
        self.stack_offset = self
            .stack_offset
            .min(self.pad.calculator().depth().saturating_sub(1));
    }
    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Input => Focus::Stack,
            Focus::Stack => Focus::History,
            Focus::History => Focus::Result,
            Focus::Result => Focus::Input,
        };
        if self.focus == Focus::History {
            self.show_history = true;
        }
        if self.focus == Focus::Stack {
            self.show_history = false;
        }
    }
    pub fn toggle_command(&mut self) {
        self.command_mode = !self.command_mode;
        self.focus = Focus::Input;
    }
    pub fn scroll(&mut self, delta: isize) {
        match self.focus {
            Focus::Stack => {
                self.stack_offset = self
                    .stack_offset
                    .saturating_add_signed(delta)
                    .min(self.pad.calculator().depth().saturating_sub(1))
            }
            Focus::History => {
                self.history_offset = self
                    .history_offset
                    .saturating_add_signed(delta)
                    .min(self.journal.len().saturating_sub(1))
            }
            Focus::Result => {
                self.value_offset = self
                    .value_offset
                    .saturating_add_signed(delta)
                    .min(self.result().chars().count().saturating_sub(1))
            }
            Focus::Input => {}
        }
    }
    pub fn recall(&mut self) {
        if let Some(entry) = self.journal.get(self.history_offset) {
            self.command = entry.source.clone();
            self.cursor = self.command.chars().count();
            let _ = self.pad.handle(Key::Cancel);
            self.command_mode = true;
            self.focus = Focus::Input;
            self.message = "Recalled into input - press Enter to execute".to_owned();
            self.failed = false;
        }
    }
    pub fn insert(&mut self, character: char) {
        if character.is_control() || self.command.chars().count() >= 4096 {
            return;
        }
        let index = self
            .command
            .char_indices()
            .nth(self.cursor)
            .map_or(self.command.len(), |(index, _)| index);
        self.command.insert(index, character);
        self.cursor += 1;
    }
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.delete();
        }
    }
    pub fn delete(&mut self) {
        if let Some((index, _)) = self.command.char_indices().nth(self.cursor) {
            self.command.remove(index);
        }
    }
    pub fn complete(&mut self) {
        let before: String = self.command.chars().take(self.cursor).collect();
        let start = before
            .rfind(char::is_whitespace)
            .map_or(0, |index| index + 1);
        let prefix = &before[start..];
        let words: Vec<_> = workflow::completions(self.pad.calculator())
            .into_iter()
            .filter(|word| word.starts_with(prefix))
            .collect();
        if words.len() == 1 {
            let end = before.len();
            self.command.replace_range(start..end, &words[0]);
            self.cursor = before[..start].chars().count() + words[0].chars().count();
        } else {
            self.message = if words.is_empty() {
                "No completion".to_owned()
            } else {
                words.into_iter().take(12).collect::<Vec<_>>().join("  ")
            };
            self.failed = false;
        }
    }
    pub fn key(&mut self, id: &str, navigation: bool, fallback: Option<Key>) -> Outcome {
        let layer = if navigation {
            "nav"
        } else {
            self.mapping.layer()
        };
        let binding = self.mapping.config.binding(layer, id).map(str::to_owned);
        let editing = binding.is_none()
            && (matches!(
                fallback,
                Some(Key::Digit(_) | Key::Decimal | Key::Backspace | Key::Cancel)
            ) || (!self.pad.pending().is_empty()
                && matches!(fallback, Some(Key::ToggleSign | Key::Undo))));
        let mut word = binding.unwrap_or_else(|| {
            match fallback {
                Some(Key::Enter) => {
                    if self.pad.pending().is_empty() {
                        "dup"
                    } else {
                        ""
                    }
                }
                Some(Key::Add) => "+",
                Some(Key::Subtract) => "-",
                Some(Key::Multiply) => "*",
                Some(Key::Divide) => "/",
                Some(Key::Undo) => ":undo",
                Some(Key::Redo) => ":redo",
                Some(Key::Clear) => "clear",
                Some(Key::ToggleSign) => "neg",
                Some(Key::Swap) => "swap",
                _ => "",
            }
            .to_owned()
        });
        if let Some(name) = word.strip_prefix("@shortcut:")
            && let Some(expression) = self.mapping.config.shortcuts.get(name)
        {
            word = expression.clone();
        }
        let editing = editing
            || matches!(word.as_str(), "@cancel" | "@backspace")
            || (!self.pad.pending().is_empty() && matches!(word.as_str(), "@sign" | "@undo"));
        if word == "@enter" {
            word = if self.pad.pending().is_empty() {
                "dup"
            } else {
                ""
            }
            .to_owned();
        }
        if word == "@swap" {
            word = "swap".to_owned();
        }
        let source = if word.starts_with('@') || word.starts_with(':') {
            word.clone()
        } else {
            format!("{} {word}", self.pad.pending()).trim().to_owned()
        };
        match self.mapping.handle(&mut self.pad, id, navigation, fallback) {
            Ok(Outcome::Quit) => Outcome::Quit,
            Ok(Outcome::Continue) => {
                if !editing && id != "layer" && !word.starts_with("@layer:") {
                    self.record(source, Ok(self.result()));
                }
                Outcome::Continue
            }
            Err(error) => {
                self.record(source, Err(error));
                Outcome::Continue
            }
        }
    }
    pub fn submit(&mut self) -> Outcome {
        let source = self.command.trim().to_owned();
        if source.is_empty() {
            return Outcome::Continue;
        }
        if matches!(source.as_str(), ":quit" | ":exit") {
            return Outcome::Quit;
        }
        self.history.record(&source);
        let result = self.execute(&source);
        if result.is_ok() {
            self.command.clear();
            self.cursor = 0;
        }
        self.mapping.config.capture(self.pad.calculator());
        self.record(source, result);
        Outcome::Continue
    }
    fn execute(&mut self, source: &str) -> Result<String, String> {
        match source {
            ":undo" | "@undo" => { self.pad.handle(Key::Undo)?; return Ok(self.result()); }
            ":redo" | "@redo" => { self.pad.handle(Key::Redo)?; return Ok(self.result()); }
            ":stack" => { self.focus = Focus::Stack; self.show_history = false; return Ok("Stack focused".to_owned()); }
            ":history" => { self.focus = Focus::History; self.show_history = true; return Ok("History focused".to_owned()); }
            ":help" => return Ok("F3 focus | F4 lower tab | F5 command/numeric | Tab complete | Enter recall in history | Esc exit".to_owned()),
            ":save" => { storage::save(&storage::default_register_path(), self.pad.calculator().registers()).map_err(|e| e.to_string())?; return Ok("Saved registers".to_owned()); }
            ":load" => { let values = storage::load(&storage::default_register_path()).map_err(|e| e.to_string())?; self.pad.calculator_mut().restore_registers(values).map_err(|e| e.to_string())?; return Ok("Loaded registers".to_owned()); }
            _ => {}
        }
        if let Some(action) = source.strip_prefix('@') {
            let (id, fallback) = match action {
                "enter" => ("enter", Key::Enter),
                "sign" => ("s", Key::ToggleSign),
                "clear" => ("c", Key::Clear),
                "swap" => ("tab", Key::Swap),
                "cancel" => ("delete", Key::Cancel),
                "backspace" => ("backspace", Key::Backspace),
                _ => return Err("Recall this binding by its macro/expression instead".to_owned()),
            };
            let _ = id;
            self.pad.handle(fallback)?;
            return Ok(self.result());
        }
        if workflow::is_command(source) {
            return workflow::command(
                self.pad.calculator_mut(),
                &mut self.mapping.config,
                &mut self.history,
                source,
            )
            .map(|lines| lines.join(" | "));
        }
        if programming::is_command(source) {
            return programming::command(self.pad.calculator_mut(), source)
                .map(|lines| lines.join(" | "));
        }
        if source == ":settings"
            || [":precision ", ":round ", ":mode ", ":display "]
                .iter()
                .any(|prefix| source.starts_with(prefix))
        {
            return workflow::configure_numeric(
                self.pad.calculator_mut(),
                &mut self.mapping.config,
                source,
            );
        }
        self.pad.execute_expression(source)?;
        Ok(self.result())
    }
    pub fn render(&self, width: usize, height: usize) -> Vec<Row> {
        let width = width.min(400);
        let height = height.min(120);
        let mut rows = Vec::new();
        let mut push = |text: String, tone| {
            rows.push(Row {
                text: clip(&text, width),
                tone,
            })
        };
        if width < 32 || height < 13 {
            push(
                "rpning - enlarge terminal (32 x 13 minimum)".to_owned(),
                Tone::Error,
            );
            push(format!("X = {}", self.result()), Tone::Result);
            push("Esc exits safely".to_owned(), Tone::Muted);
            rows.truncate(height);
            return rows;
        }
        push(
            format!(
                "  rpning  •  calculator                 FOCUS: {:?}",
                self.focus
            ),
            Tone::Accent,
        );
        push("─".repeat(width), Tone::Accent);
        push(
            format!(
                "  RESULT X   [{}]  ←/→ scroll when Result is focused",
                self.mapping.config.display.name()
            ),
            Tone::Muted,
        );
        let result = self.result();
        let value: String = result
            .chars()
            .skip(self.value_offset)
            .take(width.saturating_sub(4))
            .collect();
        let more = result.chars().count() > self.value_offset + value.chars().count();
        push(
            format!(
                "  {}{value}{}",
                if self.value_offset > 0 { "‹" } else { "" },
                if more { "…" } else { "" }
            ),
            Tone::Result,
        );
        let input = if self.command_mode {
            let mut chars: Vec<_> = self.command.chars().collect();
            chars.insert(self.cursor.min(chars.len()), '│');
            let start = self.cursor.saturating_sub(width.saturating_sub(16));
            format!(
                "  CMD › {}",
                chars.into_iter().skip(start).collect::<String>()
            )
        } else {
            format!("  NUM › {}", self.pad.pending())
        };
        push(input, Tone::Input);
        push(
            format!(
                "  {}{}",
                if self.failed { "ERROR: " } else { "" },
                self.message
            ),
            if self.failed {
                Tone::Error
            } else {
                Tone::Muted
            },
        );
        push("─".repeat(width), Tone::Accent);
        let wide = width >= 90;
        let split = width / 3;
        let stack_title = format!(
            " STACK [{}]{}",
            self.pad.calculator().depth(),
            if self.focus == Focus::Stack {
                " [FOCUS]"
            } else {
                ""
            }
        );
        let history_title = format!(" HISTORY [{}] newest first", self.journal.len());
        push(
            if wide {
                format!("{}│{}", pad(&stack_title, split), history_title)
            } else if self.show_history {
                history_title
            } else {
                stack_title
            },
            Tone::Accent,
        );
        let count = height.saturating_sub(11);
        for row in 0..count {
            let stack_index = self.stack_offset + row;
            let stack_line = self
                .pad
                .calculator()
                .stack()
                .iter()
                .rev()
                .nth(stack_index)
                .map(|value| {
                    let label = match stack_index {
                        0 => "X".to_owned(),
                        1 => "Y".to_owned(),
                        2 => "Z".to_owned(),
                        _ => stack_index.to_string(),
                    };
                    format!(
                        " {label:>3}  {}",
                        self.mapping
                            .config
                            .display
                            .format(value, self.pad.calculator().settings().rounding)
                    )
                })
                .unwrap_or_default();
            let history_line = self
                .journal
                .get(self.history_offset + row)
                .map(|entry| {
                    format!(
                        "{} {}  {} {}",
                        if row == 0 && self.focus == Focus::History {
                            "›"
                        } else {
                            " "
                        },
                        entry.source,
                        if entry.failed { "!" } else { "→" },
                        entry.result
                    )
                })
                .unwrap_or_default();
            push(
                if wide {
                    format!(
                        "{}│{}",
                        pad(&stack_line, split),
                        clip(&history_line, width - split - 1)
                    )
                } else if self.show_history {
                    history_line
                } else {
                    stack_line
                },
                Tone::Normal,
            );
        }
        push("─".repeat(width), Tone::Accent);
        let settings = self.pad.calculator().settings();
        push(
            format!(
                " {} | {} bits | {} | layer={} | F2 layer F3 focus F4 tab F5 command",
                if settings.binary_mode {
                    "BINARY"
                } else {
                    "EXACT"
                },
                settings.bits,
                settings.rounding.name(),
                self.mapping.layer()
            ),
            Tone::Muted,
        );
        push(" Arrows/PgUp/PgDn browse focused pane • Enter recalls history • Ctrl+Z/Y undo/redo • Esc exits".to_owned(), Tone::Muted);
        rows.truncate(height);
        rows
    }
}

fn clip(text: &str, width: usize) -> String {
    let chars: Vec<_> = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    if chars.len() <= width {
        return chars.into_iter().collect();
    }
    if width == 0 {
        return String::new();
    }
    chars
        .into_iter()
        .take(width - 1)
        .chain(std::iter::once('…'))
        .collect()
}
fn pad(text: &str, width: usize) -> String {
    let mut text = clip(text, width);
    text.extend(std::iter::repeat_n(' ', width - text.chars().count()));
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ui() -> Tui {
        Tui::new(Calculator::new(), Config::default())
    }
    #[test]
    fn keypad_journal_errors_and_recall_preserve_state() {
        let mut ui = ui();
        for key in [
            Key::Digit(1),
            Key::Digit(2),
            Key::Enter,
            Key::Digit(3),
            Key::Add,
        ] {
            ui.key("", false, Some(key));
        }
        assert_eq!(ui.result(), "15");
        assert_eq!(ui.journal.len(), 2);
        ui.key("", false, Some(Key::Digit(0)));
        ui.key("", false, Some(Key::Divide));
        assert!(ui.failed);
        assert_eq!(ui.result(), "15");
        assert_eq!(ui.pad.pending(), "0");
        ui.focus = Focus::History;
        ui.recall();
        assert!(ui.command_mode);
        assert_eq!(ui.command, "0 /");
        assert_eq!(ui.result(), "15");
    }
    #[test]
    fn commands_macros_completion_settings_and_undo_share_the_engine() {
        let mut ui = ui();
        ui.command = ": square dup * ;".to_owned();
        ui.submit();
        ui.command = "5 squa".to_owned();
        ui.cursor = 6;
        ui.complete();
        assert_eq!(ui.command, "5 square");
        ui.submit();
        assert_eq!(ui.result(), "25");
        ui.command = ":display hex".to_owned();
        ui.submit();
        assert_eq!(ui.result(), "0x19");
        ui.command = ":undo".to_owned();
        ui.submit();
        assert_eq!(ui.pad.calculator().depth(), 0);
        ui.command = ":redo".to_owned();
        ui.submit();
        assert_eq!(ui.result(), "0x19");
    }
    #[test]
    fn frames_resize_without_overflow_and_long_values_scroll_losslessly() {
        let mut ui = ui();
        ui.command = "1 200 shl".to_owned();
        ui.submit();
        let original = ui.result();
        for (w, h) in [(0, 0), (12, 5), (40, 15), (100, 30), (200, 60)] {
            let frame = ui.render(w, h);
            assert!(frame.len() <= h);
            assert!(frame.iter().all(|row| row.text.chars().count() <= w));
        }
        let wide = ui
            .render(100, 30)
            .iter()
            .map(|r| r.text.clone())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(wide.contains("STACK"));
        assert!(wide.contains("HISTORY"));
        ui.focus = Focus::Result;
        ui.scroll(12);
        assert_eq!(ui.value_offset, 12);
        assert_eq!(ui.result(), original);
    }
    #[test]
    fn journal_is_bounded_and_command_editing_handles_unicode() {
        let mut ui = ui();
        for _ in 0..300 {
            ui.command = "1".to_owned();
            ui.submit();
        }
        assert_eq!(ui.journal.len(), 256);
        ui.command.clear();
        ui.cursor = 0;
        ui.insert('á');
        ui.insert('界');
        ui.backspace();
        assert_eq!(ui.command, "á");
        ui.cursor = 0;
        ui.delete();
        assert!(ui.command.is_empty());
    }

    #[test]
    fn pending_sign_and_custom_enter_are_edit_or_exact_recall_actions() {
        let mut ui = ui();
        ui.key("5", false, Some(Key::Digit(5)));
        ui.mapping.config.bind("base", "s", "@sign").unwrap();
        ui.key("s", false, Some(Key::ToggleSign));
        assert_eq!(ui.pad.pending(), "-5");
        assert!(ui.journal.is_empty());
        ui.mapping.config.bind("base", "enter", "@enter").unwrap();
        ui.key("enter", false, Some(Key::Enter));
        assert_eq!(ui.journal[0].source, "-5");
        ui.key("2", false, Some(Key::Digit(2)));
        ui.recall();
        assert!(ui.pad.pending().is_empty());
        assert_eq!(ui.command, "-5");
        assert_eq!(ui.result(), "-5");
    }
}
