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
    Output,
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
    pub output: Vec<String>,
    pub output_title: String,
    pub output_offset: usize,
    pub show_output: bool,
    viewport_width: usize,
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
            output: Vec::new(),
            output_title: "Command output".to_owned(),
            output_offset: 0,
            show_output: false,
            viewport_width: 100,
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
            Focus::Result => Focus::Output,
            Focus::Output => Focus::Input,
        };
        self.show_output = self.focus == Focus::Output;
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
            Focus::Output => {
                self.output_offset = self.output_offset.saturating_add_signed(delta).min(
                    self.output_lines(self.viewport_width)
                        .len()
                        .saturating_sub(1),
                );
            }
            Focus::Input => {}
        }
    }
    pub fn resize(&mut self, width: usize) {
        let width = width.clamp(1, 400);
        if width != self.viewport_width {
            self.viewport_width = width;
            self.output_offset = 0;
        }
    }
    pub fn focus_output(&mut self) {
        self.focus = Focus::Output;
        self.show_output = true;
    }
    fn output_lines(&self, width: usize) -> Vec<String> {
        let width = width.saturating_sub(2).max(1);
        self.output
            .iter()
            .flat_map(|line| {
                let chars: Vec<_> = line
                    .chars()
                    .map(|c| if c.is_control() { ' ' } else { c })
                    .collect();
                if chars.is_empty() {
                    vec![String::new()]
                } else {
                    chars
                        .chunks(width)
                        .map(|part| part.iter().collect())
                        .collect()
                }
            })
            .collect()
    }
    fn show_lines(&mut self, title: &str, lines: Vec<String>) -> String {
        self.output_title = title.to_owned();
        self.output = if lines.is_empty() {
            vec!["(no entries)".to_owned()]
        } else {
            lines
        };
        self.output_offset = 0;
        self.show_output = true;
        format!(
            "{title} - F6 focuses output; arrows/PageUp/PageDown scroll; F4 returns to stack/history"
        )
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
        if self.cursor == self.command.chars().count()
            && workflow::setting_help(self.pad.calculator(), &self.mapping.config, &self.command)
                .is_some()
        {
            self.command.push(' ');
            self.cursor += 1;
        }
        let before: String = self.command.chars().take(self.cursor).collect();
        let start = before
            .rfind(char::is_whitespace)
            .map_or(0, |index| index + 1);
        let words = workflow::completion_candidates(self.pad.calculator(), &before);
        if words.len() == 1 {
            let end = self.command[before.len()..]
                .find(char::is_whitespace)
                .map_or(self.command.len(), |offset| before.len() + offset);
            self.command.replace_range(start..end, &words[0]);
            self.cursor = before[..start].chars().count() + words[0].chars().count();
            self.show_output = false;
        } else {
            self.message = if words.is_empty() {
                "No completion".to_owned()
            } else {
                self.show_lines("Completions", words)
            };
            self.failed = false;
        }
    }
    pub fn key(&mut self, id: &str, navigation: bool, fallback: Option<Key>) -> Outcome {
        if id != "layer" {
            self.show_output = false;
        }
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
        self.show_output = false;
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
            ":undo" | "@undo" => {
                self.pad.handle(Key::Undo)?;
                return Ok(self.result());
            }
            ":redo" | "@redo" => {
                self.pad.handle(Key::Redo)?;
                return Ok(self.result());
            }
            ":stack" => {
                self.focus = Focus::Stack;
                self.show_history = false;
                return Ok("Stack focused".to_owned());
            }
            ":history" => {
                self.focus = Focus::History;
                self.show_history = true;
                return Ok("History focused".to_owned());
            }
            ":registers" => {
                let lines = self
                    .pad
                    .calculator()
                    .registers()
                    .map(|(name, value)| {
                        format!(
                            "{name} = {}",
                            self.mapping
                                .config
                                .display
                                .format(value, self.pad.calculator().settings().rounding)
                        )
                    })
                    .collect();
                return Ok(self.show_lines("Registers", lines));
            }
            ":save" => {
                storage::save(
                    &storage::default_register_path(),
                    self.pad.calculator().registers(),
                )
                .map_err(|e| e.to_string())?;
                return Ok("Saved registers".to_owned());
            }
            ":load" => {
                let values =
                    storage::load(&storage::default_register_path()).map_err(|e| e.to_string())?;
                self.pad
                    .calculator_mut()
                    .restore_registers(values)
                    .map_err(|e| e.to_string())?;
                return Ok("Loaded registers".to_owned());
            }
            _ => {}
        }
        if source.split_whitespace().next() == Some(":help") {
            let topic = source.strip_prefix(":help").unwrap().trim();
            let text = match topic {
                "" => {
                    "TUI: F1 input | F2 layer | F3 focus | F4 stack/history | F5 numeric/command | F6 output\nEnter in History recalls; Enter in input executes. Ctrl+Z/Y undo/redo; Esc exits.\n:help tui - full TUI guide\n:help program - full program reference\n:help layers - Base/Alt/nav mappings and customization\n:help settings | :help math | :help stack | :help registers\n:help macros | :help scripts | :help persistence\n:display / :mode / :precision / :round - current value and allowed values\n:bindings - configured mappings; :history TEXT - saved command search"
                }
                "tui" => include_str!("../docs/TUI.md"),
                "program" => include_str!("../docs/PROGRAM_GUIDE.md"),
                "layers" => guide_section(include_str!("../docs/TUI.md"), "Keypad layers"),
                "settings" => guide_section(
                    include_str!("../docs/PROGRAM_GUIDE.md"),
                    "Settings and display",
                ),
                "math" => guide_section(
                    include_str!("../docs/PROGRAM_GUIDE.md"),
                    "Mathematics reference",
                ),
                "stack" => guide_section(
                    include_str!("../docs/PROGRAM_GUIDE.md"),
                    "Stack and transactions",
                ),
                "registers" => {
                    guide_section(include_str!("../docs/PROGRAM_GUIDE.md"), "Named registers")
                }
                "macros" => guide_section(
                    include_str!("../docs/PROGRAM_GUIDE.md"),
                    "Macros and programming",
                ),
                "scripts" => guide_section(
                    include_str!("../docs/PROGRAM_GUIDE.md"),
                    "Scripts and automation",
                ),
                "persistence" => guide_section(
                    include_str!("../docs/PROGRAM_GUIDE.md"),
                    "Persistence and files",
                ),
                _ => return Err("Unknown help topic; use :help".to_owned()),
            };
            return Ok(self.show_lines(source, text.lines().map(str::to_owned).collect()));
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
            let lines = workflow::command(
                self.pad.calculator_mut(),
                &mut self.mapping.config,
                &mut self.history,
                source,
            )?;
            return Ok(self.show_lines(source, lines));
        }
        if programming::is_command(source) {
            let lines = programming::command(self.pad.calculator_mut(), source)?;
            return Ok(self.show_lines(source, lines));
        }
        if workflow::is_numeric_command(source) {
            let message = workflow::configure_numeric(
                self.pad.calculator_mut(),
                &mut self.mapping.config,
                source,
            )?;
            if message.contains('\n') {
                return Ok(self.show_lines(source, message.lines().map(str::to_owned).collect()));
            }
            return Ok(message);
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
        let hints = if self.command_mode {
            workflow::setting_help(
                self.pad.calculator(),
                &self.mapping.config,
                self.command.split_whitespace().next().unwrap_or(""),
            )
            .unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut hint_rows = 0;
        // Leave room for at least two lower-pane rows in small terminals.
        for hint in hints.iter().take(height.saturating_sub(13).min(3)) {
            let chars: Vec<_> = hint.chars().collect();
            for part in chars.chunks(width.saturating_sub(2).max(1)) {
                if hint_rows >= height.saturating_sub(13).min(5) {
                    break;
                }
                push(
                    format!("  {}", part.iter().collect::<String>()),
                    Tone::Muted,
                );
                hint_rows += 1;
            }
        }
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
            if self.show_output {
                format!(
                    " OUTPUT: {} [F6 focus, F4 stack/history]",
                    self.output_title
                )
            } else if wide {
                format!("{}│{}", pad(&stack_title, split), history_title)
            } else if self.show_history {
                history_title
            } else {
                stack_title
            },
            Tone::Accent,
        );
        let count = height.saturating_sub(11 + hint_rows);
        let output_lines = if self.show_output {
            self.output_lines(width)
        } else {
            Vec::new()
        };
        let output_start = self.output_offset.min(output_lines.len().saturating_sub(1));
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
                if self.show_output {
                    format!(
                        " {}",
                        output_lines
                            .get(output_start + row)
                            .map_or("", String::as_str)
                    )
                } else if wide {
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
                " {} | {} bits | {} | layer={} | F2 layer F3 focus F4 tab F5 command F6 output",
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

fn guide_section<'a>(guide: &'a str, title: &str) -> &'a str {
    let heading = format!("## {title}");
    guide
        .find(&heading)
        .map(|start| {
            let section = &guide[start..];
            section.find("\n## ").map_or(section, |end| &section[..end])
        })
        .unwrap_or("Help section unavailable; use :help program")
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

    #[test]
    fn settings_discovery_preserves_values_pending_input_and_numeric_history() {
        let mut ui = ui();
        ui.pad.execute_expression("1/3").unwrap();
        ui.pad.handle(Key::Digit(7)).unwrap();
        ui.command_mode = true;
        ui.command = ":display".to_owned();
        ui.cursor = ui.command.len();
        let frame = ui
            .render(100, 30)
            .into_iter()
            .map(|r| r.text)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(frame.contains("current = auto"));
        assert!(frame.contains("fixed:N"));
        ui.complete();
        assert_eq!(ui.output.len(), 135);
        assert!(ui.output.contains(&"fixed:128".to_owned()));
        assert_eq!(ui.command, ":display ");
        ui.submit();
        assert!(ui.show_output && !ui.failed);
        assert!(ui.output[0].contains("current = auto"));
        assert_eq!(ui.pad.pending(), "7");
        assert_eq!(ui.result(), "1/3");
        ui.command = ":precision".to_owned();
        ui.submit();
        assert!(ui.output[0].contains("53 bits"));
        ui.pad.handle(Key::Cancel).unwrap();
        assert!(ui.pad.calculator_mut().undo());
        assert_eq!(ui.pad.calculator().depth(), 0);
        assert!(!ui.pad.calculator_mut().undo());
    }

    #[test]
    fn output_wraps_and_scrolls_without_losing_command_or_calculator_state() {
        let mut ui = ui();
        ui.command = ":display".to_owned();
        ui.cursor = ui.command.len();
        ui.complete();
        ui.focus_output();
        ui.resize(40);
        ui.scroll(100_000);
        let frame = ui
            .render(40, 15)
            .into_iter()
            .map(|r| r.text)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(frame.contains("fixed:128"), "{frame}");
        assert_eq!(ui.command, ":display ");
        assert_eq!(ui.pad.calculator().depth(), 0);
        ui.resize(100);
        assert_eq!(ui.output_offset, 0);
        ui.command = ":bindings".to_owned();
        ui.submit();
        assert!(ui.output.iter().any(|line| line == "alt.7 = dup *"));
        ui.command = "123 sto:test".to_owned();
        ui.submit();
        ui.command = ":registers".to_owned();
        ui.submit();
        assert_eq!(ui.output, ["test = 123"]);
        assert_eq!(ui.result(), "123");
    }

    #[test]
    fn embedded_guides_cover_all_topics_and_accept_crlf_checkouts() {
        let mut ui = ui();
        for topic in [
            "",
            "tui",
            "program",
            "layers",
            "settings",
            "math",
            "stack",
            "registers",
            "macros",
            "scripts",
            "persistence",
        ] {
            ui.command = format!(":help {topic}");
            ui.submit();
            assert!(!ui.failed && ui.output.len() > 3, "{topic}");
            assert!(
                !ui.output
                    .iter()
                    .any(|line| line.contains("Help section unavailable"))
            );
        }
        let guide = include_str!("../docs/PROGRAM_GUIDE.md")
            .replace("\r\n", "\n")
            .replace('\n', "\r\n");
        let chapter = guide_section(&guide, "Macros and programming");
        assert!(chapter.contains("let:NAME") && chapter.contains("while"));
        assert!(!chapter.contains("## Scripts and automation"));
    }
}
