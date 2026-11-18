use std::error::Error;
use std::fmt;

use crate::{
    CalcError::UnknownToken,
    number::{Number, parse_literal},
};

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

        for token in input.split_whitespace() {
            match token {
                "+" | "-" | "*" | "/" => {
                    require_depth(&working, 2, token)?;

                    let len = working.len();
                    let x = working.remove(len - 1);
                    let y = working.remove(len - 2);

                    if token == "/" && x == Number::from(0) {
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
    use super::Calculator;

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

        assert_eq!(calc.depth(), 0);
    }
}
