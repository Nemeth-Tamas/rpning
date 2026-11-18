use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::number::{Number, parse_literal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    StackUnderflow {
        operator: String,
        required: usize,
        available: usize,
    },
    DivisionByZero,
    UnknownToken(String),
    InvalidStackIndex {
        operator: String,
        value: String,
    },
    StackIndexOutOfRange {
        operator: String,
        index: usize,
        depth: usize,
    },
    InvalidRegisterName(String),
    UnknownRegister(String),
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StackUnderflow {
                operator,
                required,
                available,
            } => write!(
                f,
                "stack underflow in '{operator}': need {required}, have {available}"
            ),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::UnknownToken(token) => write!(f, "unknown token: '{token}'"),
            Self::InvalidStackIndex { operator, value } => write!(
                f,
                "invalid index for '{operator}': '{value}' must be a nonnegative integer that fits usize"
            ),
            Self::StackIndexOutOfRange {
                operator,
                index,
                depth,
            } => write!(
                f,
                "index {index} out of range for '{operator}': stack has {depth} selectable items"
            ),
            Self::InvalidRegisterName(name) => write!(
                f,
                "invalid register name '{name}': use 1-64 ASCII letters, digits or underscores, starting with a letter or underscore"
            ),
            Self::UnknownRegister(name) => {
                write!(f, "unknown register: '{name}'")
            }
        }
    }
}

impl Error for CalcError {}

const HISTORY_LIMIT: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    stack: Vec<Number>,
    registers: BTreeMap<String, Number>,
}

#[derive(Debug, Default, Clone)]
pub struct Calculator {
    stack: Vec<Number>,
    registers: BTreeMap<String, Number>,
    undo_history: Vec<Snapshot>,
    redo_history: Vec<Snapshot>,
}

impl Calculator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Values are stored bottom-to-top. X is the final element.
    pub fn stack(&self) -> &[Number] {
        &self.stack
    }

    pub fn top(&self) -> Option<&Number> {
        self.stack.last()
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Inspect one named register without modifying the stack.
    pub fn register(&self, name: &str) -> Option<&Number> {
        self.registers.get(name)
    }

    /// Iterate over registers in alphabetical order.
    pub fn registers(&self) -> impl Iterator<Item = (&str, &Number)> {
        self.registers
            .iter()
            .map(|(name, value)| (name.as_str(), value))
    }

    /// Restore the previous successful calculator state.
    ///
    /// Returns false if there is nothing left to undo.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo_history.pop() else {
            return false;
        };

        let current = Snapshot {
            stack: std::mem::replace(&mut self.stack, previous.stack),
            registers: std::mem::replace(&mut self.registers, previous.registers),
        };

        push_bounded(&mut self.redo_history, current);
        true
    }

    /// Restore a stack state previously reverted by undo.
    ///
    /// Returns false if there is nothing left to redo.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo_history.pop() else {
            return false;
        };

        let current = Snapshot {
            stack: std::mem::replace(&mut self.stack, next.stack),
            registers: std::mem::replace(&mut self.registers, next.registers),
        };

        push_bounded(&mut self.undo_history, current);
        true
    }

    /// Evaluate a complete input line atomically.
    ///
    /// If any token fails, neither stack nor register modifications
    /// are committed to the calculator.
    pub fn evaluate(&mut self, input: &str) -> Result<(), CalcError> {
        let mut working = self.stack.clone();
        let mut working_registers = self.registers.clone();

        for token in tokenize(input) {
            match token {
                "+" | "-" | "*" | "/" => {
                    require_depth(&working, 2, token)?;

                    let len = working.len();
                    let x = working.remove(len - 1);
                    let y = working.remove(len - 2);

                    if token == "/" && x == 0 {
                        return Err(CalcError::DivisionByZero);
                    }

                    let result = match token {
                        "+" => y + x,
                        "-" => y - x,
                        "*" => y * x,
                        "/" => y / x,
                        _ => return Err(CalcError::UnknownToken(token.to_owned())),
                    };

                    working.push(result);
                }

                "dup" => {
                    require_depth(&working, 1, token)?;

                    let len = working.len();
                    let value = working[len - 1].clone();
                    working.push(value);
                }

                "drop" => {
                    require_depth(&working, 1, token)?;
                    working.pop();
                }

                "swap" => {
                    require_depth(&working, 2, token)?;

                    let len = working.len();
                    working.swap(len - 1, len - 2);
                }

                "over" => {
                    require_depth(&working, 2, token)?;

                    let len = working.len();
                    let value = working[len - 2].clone();
                    working.push(value);
                }

                "rot" => {
                    require_depth(&working, 3, token)?;

                    let len = working.len();
                    working[len - 3..].rotate_left(1);
                }

                "nip" => {
                    require_depth(&working, 2, token)?;

                    let len = working.len();
                    working.remove(len - 2);
                }

                "tuck" => {
                    require_depth(&working, 2, token)?;

                    let len = working.len();
                    let value = working[len - 1].clone();
                    working.insert(len - 2, value);
                }

                "clear" => {
                    working.clear();
                }

                "depth" => {
                    working.push(Number::from(working.len()));
                }

                "pick" | "roll" => {
                    require_depth(&working, 1, token)?;

                    // The top item is an index, not a selectable value.
                    let raw_index = &working[working.len() - 1];
                    let index =
                        usize::try_from(raw_index).map_err(|_| CalcError::InvalidStackIndex {
                            operator: token.to_owned(),
                            value: raw_index.to_string(),
                        })?;

                    let available = working.len() - 1;

                    if index >= available {
                        return Err(CalcError::StackIndexOutOfRange {
                            operator: token.to_owned(),
                            index,
                            depth: available,
                        });
                    }

                    // The index has been validated before modifying the stack.
                    working.pop();

                    let position = working.len() - 1 - index;

                    let value = if token == "pick" {
                        working[position].clone()
                    } else {
                        working.remove(position)
                    };

                    working.push(value);
                }

                _ if token.starts_with("sto:") => {
                    let name = &token[4..];
                    validate_register_name(name)?;
                    require_depth(&working, 1, token)?;

                    let value = working[working.len() - 1].clone();
                    working_registers.insert(name.to_owned(), value);
                }

                _ if token.starts_with("rcl:") => {
                    let name = &token[4..];
                    validate_register_name(name)?;

                    let value = working_registers
                        .get(name)
                        .ok_or_else(|| CalcError::UnknownRegister(name.to_owned()))?
                        .clone();

                    working.push(value);
                }

                _ if token.starts_with("del:") => {
                    let name = &token[4..];
                    validate_register_name(name)?;

                    if working_registers.remove(name).is_none() {
                        return Err(CalcError::UnknownRegister(name.to_owned()));
                    }
                }

                "clearmem" => {
                    working_registers.clear();
                }

                _ => {
                    let value = parse_literal(token)
                        .ok_or_else(|| CalcError::UnknownToken(token.to_owned()))?;

                    working.push(value);
                }
            }
        }

        if working != self.stack || working_registers != self.registers {
            let previous = Snapshot {
                stack: std::mem::replace(&mut self.stack, working),
                registers: std::mem::replace(&mut self.registers, working_registers),
            };

            push_bounded(&mut self.undo_history, previous);
            self.redo_history.clear();
        }

        Ok(())
    }
}

/// Retain only the latest HISTORY_LIMIT calculator snapshots.
fn push_bounded(history: &mut Vec<Snapshot>, snapshot: Snapshot) {
    if history.len() == HISTORY_LIMIT {
        history.remove(0);
    }

    history.push(snapshot);
}

/// Validate register identifiers used in RPN instructions.
fn validate_register_name(name: &str) -> Result<(), CalcError> {
    let valid = !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && name
            .bytes()
            .skip(1)
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');

    if valid {
        Ok(())
    } else {
        Err(CalcError::InvalidRegisterName(name.to_owned()))
    }
}

/// Split a numeric literal with an attached trailing operator.
///
/// Examples:
/// "3+"    -> ["3", "+"]
/// "-2.5*" -> ["-2.5", "*"]
/// "1/3+"  -> ["1/3", "+"]
///
/// Existing exact literals are preserved. Only one trailing
/// operator is separated from a valid numeric literal.
fn tokenize(input: &str) -> impl Iterator<Item = &str> {
    input.split_whitespace().flat_map(|chunk| {
        let suffix = chunk.char_indices().last().and_then(|(index, last)| {
            let (number, operator) = chunk.split_at(index);

            if index > 0 && matches!(last, '+' | '-' | '*' | '/') && parse_literal(number).is_some()
            {
                Some((number, operator))
            } else {
                None
            }
        });

        let tokens = match suffix {
            Some((number, operator)) => [Some(number), Some(operator)],
            None => [Some(chunk), None],
        };

        tokens.into_iter().flatten()
    })
}

fn require_depth(stack: &[Number], required: usize, operator: &str) -> Result<(), CalcError> {
    if stack.len() >= required {
        Ok(())
    } else {
        Err(CalcError::StackUnderflow {
            operator: operator.to_owned(),
            required,
            available: stack.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CalcError, Calculator};

    fn assert_stack(calc: &Calculator, expected: &[&str]) {
        let actual: Vec<String> = calc.stack().iter().map(ToString::to_string).collect();
        let expected: Vec<String> = expected.iter().map(ToString::to_string).collect();

        assert_eq!(actual, expected);
    }

    #[test]
    fn basic_arithmetic() {
        let mut calc = Calculator::new();

        calc.evaluate("15 7 + 3 *").unwrap();

        assert_eq!(calc.top().unwrap().to_string(), "66");
        assert_eq!(calc.depth(), 1);
    }

    #[test]
    fn order_of_operands_is_correct() {
        let mut calc = Calculator::new();

        calc.evaluate("10 3 -").unwrap();
        assert_eq!(calc.top().unwrap().to_string(), "7");

        calc.evaluate("clear 10 4 /").unwrap();
        assert_eq!(calc.top().unwrap().to_string(), "5/2");
    }

    #[test]
    fn exact_decimal_arithmetic() {
        let mut calc = Calculator::new();

        calc.evaluate("0.1 0.2 +").unwrap();

        assert_eq!(calc.top().unwrap().to_string(), "3/10");
    }

    #[test]
    fn exact_rational_arithmetic() {
        let mut calc = Calculator::new();

        calc.evaluate("1 3 / 3 *").unwrap();

        assert_eq!(calc.top().unwrap().to_string(), "1");
    }

    #[test]
    fn attached_operator_uses_existing_stack() {
        let mut calc = Calculator::new();

        calc.evaluate("5").unwrap();
        assert_stack(&calc, &["5"]);

        calc.evaluate("3+").unwrap();
        assert_stack(&calc, &["8"]);

        calc.evaluate("2*").unwrap();
        assert_stack(&calc, &["16"]);
    }

    #[test]
    fn attached_operator_preserves_exact_numbers() {
        let mut calc = Calculator::new();

        calc.evaluate("1/3").unwrap();
        calc.evaluate("1/3+").unwrap();
        assert_stack(&calc, &["2/3"]);

        calc.evaluate("clear 0.1 0.2+").unwrap();
        assert_stack(&calc, &["3/10"]);

        calc.evaluate("clear 5 -3+").unwrap();
        assert_stack(&calc, &["2"]);

        calc.evaluate("clear 1e-3 2e-3+").unwrap();
        assert_stack(&calc, &["3/1000"]);
    }

    #[test]
    fn attached_operator_failure_is_atomic() {
        let mut calc = Calculator::new();

        calc.evaluate("5").unwrap();
        let before = calc.stack().to_vec();

        assert!(calc.evaluate("0/").is_err());
        assert_eq!(calc.stack(), before);

        assert!(calc.evaluate("3+ invalid").is_err());
        assert_eq!(calc.stack(), before);
    }

    #[test]
    fn stack_manipulation() {
        let mut calc = Calculator::new();

        calc.evaluate("10 dup *").unwrap();
        assert_eq!(calc.top().unwrap().to_string(), "100");

        calc.evaluate("2 swap").unwrap();
        assert_eq!(calc.top().unwrap().to_string(), "100");

        calc.evaluate("drop").unwrap();
        assert_eq!(calc.top().unwrap().to_string(), "2");
    }

    #[test]
    fn over_duplicates_second_item() {
        let mut calc = Calculator::new();

        calc.evaluate("3 4 over").unwrap();

        assert_stack(&calc, &["3", "4", "3"]);
    }

    #[test]
    fn rot_rotates_top_three_items() {
        let mut calc = Calculator::new();

        calc.evaluate("1 2 3 rot").unwrap();

        assert_stack(&calc, &["2", "3", "1"]);

        calc.evaluate("rot").unwrap();

        assert_stack(&calc, &["3", "1", "2"]);

        calc.evaluate("rot").unwrap();

        assert_stack(&calc, &["1", "2", "3"]);
    }

    #[test]
    fn rot_preserves_items_below_top_three() {
        let mut calc = Calculator::new();

        calc.evaluate("10 20 30 40 rot").unwrap();

        assert_stack(&calc, &["10", "30", "40", "20"]);
    }

    #[test]
    fn nip_removes_second_item() {
        let mut calc = Calculator::new();

        calc.evaluate("3 4 nip").unwrap();

        assert_stack(&calc, &["4"]);
    }

    #[test]
    fn tuck_duplicates_top_behind_second_item() {
        let mut calc = Calculator::new();

        calc.evaluate("3 4 tuck").unwrap();

        assert_stack(&calc, &["4", "3", "4"]);
    }

    #[test]
    fn new_stack_operations_work_together() {
        let mut calc = Calculator::new();

        calc.evaluate("3 4 over + nip").unwrap();

        assert_stack(&calc, &["7"]);
    }

    #[test]
    fn pick_duplicates_the_indexed_item() {
        let mut calc = Calculator::new();

        calc.evaluate("10 20 30 1 pick").unwrap();
        assert_stack(&calc, &["10", "20", "30", "20"]);

        calc.evaluate("clear 10 20 30 0 pick").unwrap();
        assert_stack(&calc, &["10", "20", "30", "30"]);

        calc.evaluate("clear 10 20 30 2 pick").unwrap();
        assert_stack(&calc, &["10", "20", "30", "10"]);
    }

    #[test]
    fn roll_moves_the_indexed_item_to_top() {
        let mut calc = Calculator::new();

        calc.evaluate("10 20 30 2 roll").unwrap();
        assert_stack(&calc, &["20", "30", "10"]);

        calc.evaluate("clear 10 20 30 1 roll").unwrap();
        assert_stack(&calc, &["10", "30", "20"]);

        calc.evaluate("clear 10 20 30 0 roll").unwrap();
        assert_stack(&calc, &["10", "20", "30"]);
    }

    #[test]
    fn depth_pushes_the_previous_stack_size() {
        let mut calc = Calculator::new();

        calc.evaluate("depth").unwrap();
        assert_stack(&calc, &["0"]);

        calc.evaluate("clear 10 20 30 depth").unwrap();
        assert_stack(&calc, &["10", "20", "30", "3"]);
    }

    #[test]
    fn indexed_operations_accept_exact_integer_values() {
        let mut calc = Calculator::new();

        calc.evaluate("10 20 30 2/2 pick").unwrap();
        assert_stack(&calc, &["10", "20", "30", "20"]);
    }

    #[test]
    fn indexed_operations_reject_invalid_values_atomically() {
        let mut calc = Calculator::new();

        calc.evaluate("10 20").unwrap();
        let before = calc.stack().to_vec();

        for expression in [
            "-1 pick",
            "1/2 pick",
            "1.5 roll",
            "18446744073709551616000000000000000 pick",
        ] {
            let error = calc.evaluate(expression).unwrap_err();

            assert!(
                matches!(error, CalcError::InvalidStackIndex { .. }),
                "unexpected error for {expression}: {error}"
            );

            assert_eq!(calc.stack(), before);
        }
    }

    #[test]
    fn indexed_operations_reject_out_of_range_indices() {
        let mut calc = Calculator::new();

        calc.evaluate("10 20").unwrap();
        let before = calc.stack().to_vec();

        for expression in ["2 pick", "99 roll"] {
            let error = calc.evaluate(expression).unwrap_err();

            assert!(
                matches!(error, CalcError::StackIndexOutOfRange { .. }),
                "unexpected error for {expression}: {error}"
            );

            assert_eq!(calc.stack(), before);
        }

        calc.evaluate("clear").unwrap();

        assert!(matches!(
            calc.evaluate("pick"),
            Err(CalcError::StackUnderflow { .. })
        ));

        assert!(matches!(
            calc.evaluate("0 roll"),
            Err(CalcError::StackIndexOutOfRange { .. })
        ));

        assert_eq!(calc.depth(), 0);
    }

    #[test]
    fn undo_and_redo_restore_completed_lines() {
        let mut calc = Calculator::new();

        assert!(!calc.undo());
        assert!(!calc.redo());

        calc.evaluate("5").unwrap();
        calc.evaluate("3+").unwrap();
        calc.evaluate("2*").unwrap();

        assert_stack(&calc, &["16"]);

        assert!(calc.undo());
        assert_stack(&calc, &["8"]);

        assert!(calc.undo());
        assert_stack(&calc, &["5"]);

        assert!(calc.undo());
        assert_stack(&calc, &[]);

        assert!(!calc.undo());

        assert!(calc.redo());
        assert_stack(&calc, &["5"]);

        assert!(calc.redo());
        assert_stack(&calc, &["8"]);

        assert!(calc.redo());
        assert_stack(&calc, &["16"]);

        assert!(!calc.redo());
    }

    #[test]
    fn clear_can_be_undone() {
        let mut calc = Calculator::new();

        calc.evaluate("10 20 30").unwrap();
        calc.evaluate("clear").unwrap();

        assert_stack(&calc, &[]);

        assert!(calc.undo());
        assert_stack(&calc, &["10", "20", "30"]);

        assert!(calc.redo());
        assert_stack(&calc, &[]);
    }

    #[test]
    fn failed_lines_do_not_enter_history() {
        let mut calc = Calculator::new();

        calc.evaluate("5").unwrap();
        calc.evaluate("3+").unwrap();

        assert!(calc.evaluate("2* invalid").is_err());
        assert_stack(&calc, &["8"]);

        assert!(calc.undo());
        assert_stack(&calc, &["5"]);

        assert!(calc.undo());
        assert_stack(&calc, &[]);

        assert!(!calc.undo());
    }

    #[test]
    fn new_calculations_invalidate_redo() {
        let mut calc = Calculator::new();

        calc.evaluate("5").unwrap();
        calc.evaluate("3+").unwrap();

        assert!(calc.undo());
        assert_stack(&calc, &["5"]);

        calc.evaluate("2+").unwrap();
        assert_stack(&calc, &["7"]);

        assert!(!calc.redo());
    }

    #[test]
    fn unchanged_lines_preserve_redo_history() {
        let mut calc = Calculator::new();

        calc.evaluate("5").unwrap();
        calc.evaluate("3+").unwrap();

        assert!(calc.undo());
        assert_stack(&calc, &["5"]);

        calc.evaluate("").unwrap();
        calc.evaluate("clear 5").unwrap();

        assert!(calc.evaluate("0/").is_err());
        assert_stack(&calc, &["5"]);

        assert!(calc.redo());
        assert_stack(&calc, &["8"]);
    }

    #[test]
    fn deleting_register_preserves_stack_and_supports_history() {
        let mut calc = Calculator::new();

        calc.evaluate("10 sto:net 20 sto:tax").unwrap();
        let before = calc.stack().to_vec();

        calc.evaluate("del:net").unwrap();

        assert_eq!(calc.stack(), before);
        assert!(calc.register("net").is_none());
        assert_eq!(calc.register("tax").unwrap().to_string(), "20");

        assert!(calc.undo());
        assert_eq!(calc.stack(), before);
        assert_eq!(calc.register("net").unwrap().to_string(), "10");

        assert!(calc.redo());
        assert_eq!(calc.stack(), before);
        assert!(calc.register("net").is_none());
    }

    #[test]
    fn clearing_memory_preserves_stack_and_supports_history() {
        let mut calc = Calculator::new();

        calc.evaluate("1 3 / sto:third 42 sto:answer").unwrap();
        let before = calc.stack().to_vec();

        calc.evaluate("clearmem").unwrap();

        assert_eq!(calc.stack(), before);
        assert_eq!(calc.registers().count(), 0);

        assert!(calc.undo());
        assert_eq!(calc.stack(), before);
        assert_eq!(calc.register("third").unwrap().to_string(), "1/3");
        assert_eq!(calc.register("answer").unwrap().to_string(), "42");

        assert!(calc.redo());
        assert_eq!(calc.registers().count(), 0);
    }

    #[test]
    fn failed_register_deletion_and_clear_are_atomic() {
        let mut calc = Calculator::new();

        calc.evaluate("10 sto:saved").unwrap();
        let before = calc.stack().to_vec();

        assert!(matches!(
            calc.evaluate("del:missing"),
            Err(CalcError::UnknownRegister(_))
        ));

        for expression in ["del:", "del:2bad"] {
            assert!(matches!(
                calc.evaluate(expression),
                Err(CalcError::InvalidRegisterName(_))
            ));
        }

        assert!(calc.evaluate("del:saved invalid").is_err());
        assert!(calc.evaluate("clearmem invalid").is_err());

        assert_eq!(calc.stack(), before);
        assert_eq!(calc.register("saved").unwrap().to_string(), "10");
    }

    #[test]
    fn clearing_empty_memory_does_not_invalidate_redo() {
        let mut calc = Calculator::new();

        calc.evaluate("5").unwrap();
        calc.evaluate("3+").unwrap();

        assert!(calc.undo());
        assert_stack(&calc, &["5"]);

        calc.evaluate("clearmem").unwrap();

        assert!(calc.redo());
        assert_stack(&calc, &["8"]);
    }

    #[test]
    fn history_is_bounded_to_128_snapshots() {
        let mut calc = Calculator::new();

        for _ in 0..140 {
            calc.evaluate("1").unwrap();
        }

        assert_eq!(calc.depth(), 140);

        for _ in 0..128 {
            assert!(calc.undo());
        }

        assert_eq!(calc.depth(), 12);
        assert!(!calc.undo());

        for _ in 0..128 {
            assert!(calc.redo());
        }

        assert_eq!(calc.depth(), 140);
        assert!(!calc.redo());
    }

    #[test]
    fn registers_store_and_recall_exact_values() {
        let mut calc = Calculator::new();

        calc.evaluate("1 3 / sto:third").unwrap();
        assert_stack(&calc, &["1/3"]);
        assert_eq!(calc.register("third").unwrap().to_string(), "1/3");

        calc.evaluate("clear rcl:third 3 *").unwrap();
        assert_stack(&calc, &["1"]);

        calc.evaluate("42 sto:Answer").unwrap();
        assert!(calc.register("answer").is_none());
        assert_eq!(calc.register("Answer").unwrap().to_string(), "42");
    }

    #[test]
    fn registers_are_visible_within_the_same_input_line() {
        let mut calc = Calculator::new();

        calc.evaluate("10 sto:a clear rcl:a 2 *").unwrap();

        assert_stack(&calc, &["20"]);
        assert_eq!(calc.register("a").unwrap().to_string(), "10");
    }

    #[test]
    fn registers_are_sorted_by_name() {
        let mut calc = Calculator::new();

        calc.evaluate("1 sto:z 2 sto:a 3 sto:m").unwrap();

        let names: Vec<&str> = calc.registers().map(|(name, _)| name).collect();

        assert_eq!(names, ["a", "m", "z"]);
    }

    #[test]
    fn register_only_changes_can_be_undone_and_redone() {
        let mut calc = Calculator::new();

        calc.evaluate("10").unwrap();
        calc.evaluate("sto:memory").unwrap();

        assert_stack(&calc, &["10"]);
        assert_eq!(calc.register("memory").unwrap().to_string(), "10");

        assert!(calc.undo());
        assert_stack(&calc, &["10"]);
        assert!(calc.register("memory").is_none());

        assert!(calc.redo());
        assert_stack(&calc, &["10"]);
        assert_eq!(calc.register("memory").unwrap().to_string(), "10");
    }

    #[test]
    fn undo_restores_overwritten_registers() {
        let mut calc = Calculator::new();

        calc.evaluate("10 sto:memory").unwrap();
        calc.evaluate("20 sto:memory").unwrap();

        assert_eq!(calc.register("memory").unwrap().to_string(), "20");

        assert!(calc.undo());
        assert_stack(&calc, &["10"]);
        assert_eq!(calc.register("memory").unwrap().to_string(), "10");

        assert!(calc.redo());
        assert_stack(&calc, &["10", "20"]);
        assert_eq!(calc.register("memory").unwrap().to_string(), "20");
    }

    #[test]
    fn register_writes_are_atomic_on_failure() {
        let mut calc = Calculator::new();

        calc.evaluate("5 sto:saved").unwrap();
        let before = calc.stack().to_vec();

        assert!(calc.evaluate("20 sto:saved rcl:missing").is_err());
        assert!(calc.evaluate("99 sto:temporary invalid").is_err());

        assert_eq!(calc.stack(), before);
        assert_eq!(calc.register("saved").unwrap().to_string(), "5");
        assert!(calc.register("temporary").is_none());

        assert!(calc.undo());
        assert_stack(&calc, &[]);
        assert!(calc.register("saved").is_none());
    }

    #[test]
    fn registers_reject_invalid_identifiers() {
        let mut calc = Calculator::new();

        calc.evaluate("10").unwrap();

        for expression in [
            "sto:",
            "rcl:",
            "sto:2bad",
            "rcl:bad-name",
            "sto:bad.name",
            "sto:é",
            "sto:abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklm",
        ] {
            assert!(
                matches!(
                    calc.evaluate(expression),
                    Err(CalcError::InvalidRegisterName(_))
                ),
                "expression: {expression}"
            );
        }

        assert_stack(&calc, &["10"]);
        assert_eq!(calc.registers().count(), 0);
    }

    #[test]
    fn missing_register_and_empty_stack_are_errors() {
        let mut calc = Calculator::new();

        assert!(matches!(
            calc.evaluate("sto:answer"),
            Err(CalcError::StackUnderflow { .. })
        ));

        assert!(matches!(
            calc.evaluate("rcl:answer"),
            Err(CalcError::UnknownRegister(_))
        ));

        assert_eq!(calc.depth(), 0);
    }

    #[test]
    fn register_changes_invalidate_redo() {
        let mut calc = Calculator::new();

        calc.evaluate("5").unwrap();
        calc.evaluate("3+").unwrap();

        assert!(calc.undo());
        assert_stack(&calc, &["5"]);

        calc.evaluate("sto:new").unwrap();

        assert!(!calc.redo());
        assert_eq!(calc.register("new").unwrap().to_string(), "5");
    }

    #[test]
    fn unchanged_register_writes_preserve_redo() {
        let mut calc = Calculator::new();

        calc.evaluate("5 sto:answer").unwrap();
        calc.evaluate("3+ sto:answer").unwrap();

        assert!(calc.undo());
        assert_eq!(calc.register("answer").unwrap().to_string(), "5");

        calc.evaluate("sto:answer").unwrap();

        assert!(calc.redo());
        assert_stack(&calc, &["8"]);
        assert_eq!(calc.register("answer").unwrap().to_string(), "8");
    }

    #[test]
    fn failed_lines_do_not_mutate_the_stack() {
        let mut calc = Calculator::new();

        calc.evaluate("5 8").unwrap();
        let before = calc.stack().to_vec();

        assert!(calc.evaluate("2 + 0 /").is_err());
        assert_eq!(calc.stack(), before);

        assert!(calc.evaluate("clear +").is_err());
        assert_eq!(calc.stack(), before);

        assert!(calc.evaluate("100 unknown").is_err());
        assert_eq!(calc.stack(), before);

        assert!(calc.evaluate("2 3 over + unknown").is_err());
        assert_eq!(calc.stack(), before);
    }

    #[test]
    fn division_by_zero_is_an_error() {
        let mut calc = Calculator::new();

        assert!(calc.evaluate("12 0 /").is_err());
        assert_eq!(calc.depth(), 0);
    }

    #[test]
    fn stack_underflow_is_an_error() {
        let mut calc = Calculator::new();

        assert!(calc.evaluate("+").is_err());
        assert!(calc.evaluate("dup").is_err());
        assert!(calc.evaluate("swap").is_err());
        assert!(calc.evaluate("over").is_err());
        assert!(calc.evaluate("rot").is_err());
        assert!(calc.evaluate("nip").is_err());
        assert!(calc.evaluate("tuck").is_err());

        assert_eq!(calc.depth(), 0);
    }

    #[test]
    fn new_stack_operations_report_correct_underflow() {
        let mut calc = Calculator::new();

        calc.evaluate("7").unwrap();

        let before = calc.stack().to_vec();

        for operator in ["over", "rot", "nip", "tuck"] {
            let error = calc.evaluate(operator).unwrap_err();

            assert!(matches!(error, CalcError::StackUnderflow { .. }));

            assert_eq!(calc.stack(), before);
        }
    }

    #[test]
    fn rot_requires_three_items() {
        let mut calc = Calculator::new();

        calc.evaluate("1 2").unwrap();

        let error = calc.evaluate("rot").unwrap_err();

        assert_eq!(
            error,
            CalcError::StackUnderflow {
                operator: "rot".to_owned(),
                required: 3,
                available: 2,
            }
        );

        assert_stack(&calc, &["1", "2"]);
    }
}
