//! Hardware-independent, transactional keypad entry. No terminal I/O lives here.

use crate::Calculator;

const MAX_ENTRY_LENGTH: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Digit(u8),
    Decimal,
    Enter,
    Add,
    Subtract,
    Multiply,
    Divide,
    Backspace,
    Cancel,
    Clear,
    ToggleSign,
    Swap,
    Undo,
    Redo,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Quit,
}

#[derive(Debug, Default)]
pub struct Keypad {
    calculator: Calculator,
    pending: String,
}

impl Keypad {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_calculator(calculator: Calculator) -> Self {
        Self {
            calculator,
            pending: String::new(),
        }
    }

    pub fn execute_expression(&mut self, expression: &str) -> Result<(), String> {
        self.execute(expression)
    }

    pub fn calculator(&self) -> &Calculator {
        &self.calculator
    }

    pub fn calculator_mut(&mut self) -> &mut Calculator {
        &mut self.calculator
    }

    pub fn pending(&self) -> &str {
        &self.pending
    }

    /// A failed operation preserves both the pending entry and calculator state.
    /// Pending-number plus operator is one calculator transaction and undo step.
    pub fn handle(&mut self, key: Key) -> Result<Outcome, String> {
        match key {
            Key::Digit(digit) => {
                if digit > 9 {
                    return Err("invalid keypad digit".to_owned());
                }
                self.append(char::from(b'0' + digit))?;
            }
            Key::Decimal => {
                if !self.pending.contains('.') {
                    let prefix = match self.pending.as_str() {
                        "" => "0.",
                        "-" => "0.",
                        _ => ".",
                    };
                    if self.pending.len() + prefix.len() > MAX_ENTRY_LENGTH {
                        return Err("pending number is limited to 4096 characters".to_owned());
                    }
                    self.pending.push_str(prefix);
                }
            }
            Key::Enter => self.execute(if self.pending.is_empty() { "dup" } else { "" })?,
            Key::Add => self.execute("+")?,
            Key::Subtract => self.execute("-")?,
            Key::Multiply => self.execute("*")?,
            Key::Divide => self.execute("/")?,
            Key::Swap => self.execute("swap")?,
            Key::Backspace => {
                self.pending.pop();
            }
            Key::Cancel => self.pending.clear(),
            Key::Clear => {
                self.calculator
                    .evaluate("clear")
                    .map_err(|e| e.to_string())?;
                self.pending.clear();
            }
            Key::ToggleSign => {
                if self.pending.is_empty() {
                    self.execute("-1 *")?;
                } else if self.pending.starts_with('-') {
                    self.pending.remove(0);
                } else {
                    if self.pending.len() == MAX_ENTRY_LENGTH {
                        return Err("pending number is limited to 4096 characters".to_owned());
                    }
                    self.pending.insert(0, '-');
                }
            }
            Key::Undo => {
                if !self.pending.is_empty() {
                    self.pending.clear();
                } else if !self.calculator.undo() {
                    return Err("nothing to undo".to_owned());
                }
            }
            Key::Redo => {
                if !self.pending.is_empty() {
                    return Err("commit or cancel the pending number before redo".to_owned());
                }
                if !self.calculator.redo() {
                    return Err("nothing to redo".to_owned());
                }
            }
            Key::Quit => return Ok(Outcome::Quit),
        }
        Ok(Outcome::Continue)
    }

    fn append(&mut self, character: char) -> Result<(), String> {
        if self.pending.len() == MAX_ENTRY_LENGTH {
            return Err("pending number is limited to 4096 characters".to_owned());
        }
        self.pending.push(character);
        Ok(())
    }

    fn execute(&mut self, operator: &str) -> Result<(), String> {
        // A trailing decimal separator means an integer, not an incomplete literal.
        let number = self.pending.trim_end_matches('.');
        if number == "-" {
            return Err("finish or cancel the pending number".to_owned());
        }
        let expression = format!("{number} {operator}");
        self.calculator
            .evaluate(&expression)
            .map_err(|e| e.to_string())?;
        self.pending.clear();
        Ok(())
    }
}

/// Translate characters reported by a terminal, independent of keyboard layout.
/// Both decimal separators are accepted only in the opt-in keypad interface.
pub fn character_key(character: char) -> Option<Key> {
    match character {
        '0'..='9' => Some(Key::Digit(character as u8 - b'0')),
        '.' | ',' => Some(Key::Decimal),
        '+' => Some(Key::Add),
        '-' => Some(Key::Subtract),
        '*' => Some(Key::Multiply),
        '/' => Some(Key::Divide),
        's' | 'S' => Some(Key::ToggleSign),
        'c' | 'C' => Some(Key::Clear),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, Keypad, MAX_ENTRY_LENGTH, Outcome, character_key};
    use crate::number::format_number;

    fn keys(pad: &mut Keypad, text: &str) {
        for character in text.chars() {
            pad.handle(character_key(character).unwrap()).unwrap();
        }
    }

    fn stack(pad: &Keypad) -> Vec<String> {
        pad.calculator().stack().iter().map(format_number).collect()
    }

    #[test]
    fn pending_entry_is_separate_and_operators_execute_immediately() {
        let mut pad = Keypad::new();
        keys(&mut pad, "12");
        assert!(stack(&pad).is_empty());
        assert_eq!(pad.pending(), "12");
        pad.handle(Key::Enter).unwrap();
        keys(&mut pad, "3+2*");
        assert_eq!(stack(&pad), ["30"]);
        assert_eq!(pad.pending(), "");
        pad.handle(Key::Undo).unwrap();
        assert_eq!(stack(&pad), ["15"]);
        pad.handle(Key::Redo).unwrap();
        assert_eq!(stack(&pad), ["30"]);
    }

    #[test]
    fn enter_commits_once_then_duplicates_and_operators_preserve_order() {
        let mut pad = Keypad::new();
        keys(&mut pad, "8");
        pad.handle(Key::Enter).unwrap();
        pad.handle(Key::Enter).unwrap();
        assert_eq!(stack(&pad), ["8", "8"]);
        keys(&mut pad, "+2/3-");
        assert_eq!(stack(&pad), ["5"]);
    }

    #[test]
    fn decimal_comma_editing_and_sign_are_exact() {
        let mut pad = Keypad::new();
        keys(&mut pad, ",12");
        pad.handle(Key::Backspace).unwrap();
        keys(&mut pad, "5.s");
        assert_eq!(pad.pending(), "-0.15");
        pad.handle(Key::Enter).unwrap();
        keys(&mut pad, "0,2+");
        assert_eq!(stack(&pad), ["0.05"]);
        pad.handle(Key::ToggleSign).unwrap();
        assert_eq!(stack(&pad), ["-0.05"]);
    }

    #[test]
    fn failed_operator_keeps_entry_stack_registers_and_history() {
        let mut pad = Keypad::new();
        pad.calculator.evaluate("7 sto:saved").unwrap();
        keys(&mut pad, "0");
        assert!(pad.handle(Key::Divide).is_err());
        assert_eq!(pad.pending(), "0");
        assert_eq!(stack(&pad), ["7"]);
        assert_eq!(
            format_number(pad.calculator.register("saved").unwrap()),
            "7"
        );
        pad.handle(Key::Backspace).unwrap();
        keys(&mut pad, "2/");
        assert_eq!(stack(&pad), ["3.5"]);
        pad.handle(Key::Undo).unwrap();
        assert_eq!(stack(&pad), ["7"]);
        pad.handle(Key::Undo).unwrap();
        assert!(stack(&pad).is_empty());
        assert!(pad.calculator.register("saved").is_none());
    }

    #[test]
    fn cancel_backspace_and_undo_do_not_drop_committed_values() {
        let mut pad = Keypad::new();
        keys(&mut pad, "5");
        pad.handle(Key::Enter).unwrap();
        pad.handle(Key::Backspace).unwrap();
        keys(&mut pad, "123");
        pad.handle(Key::Cancel).unwrap();
        keys(&mut pad, "9");
        pad.handle(Key::Undo).unwrap();
        assert_eq!(stack(&pad), ["5"]);
        assert_eq!(pad.pending(), "");
        pad.handle(Key::Clear).unwrap();
        assert!(stack(&pad).is_empty());
        pad.handle(Key::Undo).unwrap();
        assert_eq!(stack(&pad), ["5"]);
    }

    #[test]
    fn underflow_and_redo_with_pending_entry_leave_state_unchanged() {
        let mut pad = Keypad::new();
        assert!(pad.handle(Key::Enter).is_err());
        assert!(pad.handle(Key::Undo).is_err());
        assert!(pad.handle(Key::Redo).is_err());
        keys(&mut pad, "6");
        assert!(pad.handle(Key::Add).is_err());
        assert!(pad.handle(Key::Redo).is_err());
        assert_eq!(pad.pending(), "6");
        assert!(stack(&pad).is_empty());
        assert_eq!(pad.handle(Key::Quit).unwrap(), Outcome::Quit);
        assert_eq!(pad.pending(), "6");
    }

    #[test]
    fn trailing_decimal_and_sign_only_entry_have_predictable_behavior() {
        let mut pad = Keypad::new();
        keys(&mut pad, "2.");
        pad.handle(Key::Enter).unwrap();
        assert_eq!(stack(&pad), ["2"]);
        keys(&mut pad, "1s");
        pad.handle(Key::Backspace).unwrap();
        assert_eq!(pad.pending(), "-");
        assert!(pad.handle(Key::Enter).is_err());
        pad.handle(Key::Decimal).unwrap();
        assert_eq!(pad.pending(), "-0.");
        keys(&mut pad, "5+");
        assert_eq!(stack(&pad), ["1.5"]);
    }

    #[test]
    fn entry_limits_and_invalid_keys_fail_without_mutation() {
        let mut pad = Keypad::new();
        for _ in 0..MAX_ENTRY_LENGTH {
            pad.handle(Key::Digit(1)).unwrap();
        }
        let before = pad.pending().to_owned();
        for key in [Key::Digit(0), Key::Digit(10), Key::Decimal, Key::ToggleSign] {
            assert!(pad.handle(key).is_err());
            assert_eq!(pad.pending(), before);
        }
        assert_eq!(character_key('x'), None);
    }

    #[test]
    fn pending_swap_and_failed_division_remain_single_undo_transactions() {
        let mut pad = Keypad::new();
        keys(&mut pad, "3");
        pad.handle(Key::Enter).unwrap();
        keys(&mut pad, "4");
        pad.handle(Key::Swap).unwrap();
        assert_eq!(stack(&pad), ["4", "3"]);
        pad.handle(Key::Undo).unwrap();
        assert_eq!(stack(&pad), ["3"]);
        keys(&mut pad, "0");
        assert!(pad.handle(Key::Divide).is_err());
        pad.handle(Key::Cancel).unwrap();
        pad.handle(Key::Redo).unwrap();
        assert_eq!(stack(&pad), ["4", "3"]);
    }
}
