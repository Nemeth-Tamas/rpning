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
        }
    }
}

impl Error for CalcError {}

#[derive(Debug, Default, Clone)]
pub struct Calculator {
    stack: Vec<Number>,
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

    /// Evaluate a complete input line atomically.
    ///
    /// If any token fails, none of the line's modifications
    /// are committed to the calculator's stack.
    pub fn evaluate(&mut self, input: &str) -> Result<(), CalcError> {
        let mut working = self.stack.clone();

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

                _ => {
                    let value = parse_literal(token)
                        .ok_or_else(|| CalcError::UnknownToken(token.to_owned()))?;

                    working.push(value);
                }
            }
        }

        self.stack = working;
        Ok(())
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
