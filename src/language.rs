//! Small, compiled RPN language. No operating-system commands or I/O in the VM.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::Calculator;
use crate::number::{Number, parse_literal};

pub const MAX_STEPS: usize = 100_000;
pub const MAX_CALL_DEPTH: usize = 64;
const MAX_SOURCE: usize = 1024 * 1024;
const MAX_TOKENS: usize = 100_000;
const MAX_DEFINITIONS: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Push(Number),
    Word(String),
    Define(String, Arc<Definition>),
    Jump(usize),
    JumpUnless(usize),
    Repeat(usize),
    Next(usize),
    Set(String),
    Get(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Program(pub Vec<Instruction>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub description: String,
    pub source: String,
    pub program: Program,
}

pub type Definitions = BTreeMap<String, Arc<Definition>>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Text(String),
}

impl Token {
    fn word(&self) -> &str {
        match self {
            Self::Word(value) => value,
            Self::Text(_) => "",
        }
    }
    fn source(&self) -> String {
        match self {
            Self::Word(value) => value.clone(),
            Self::Text(value) => format!(
                "\"{}\"",
                value
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t")
            ),
        }
    }
}

fn lex(source: &str) -> Result<Vec<Token>, String> {
    if source.len() > MAX_SOURCE {
        return Err("source exceeds 1 MiB".to_owned());
    }
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(character) = chars.next() {
        if character.is_whitespace() {
            continue;
        }
        if character == '#' {
            for next in chars.by_ref() {
                if next == '\n' {
                    break;
                }
            }
            continue;
        }
        if character == '"' {
            let mut text = String::new();
            let mut closed = false;
            while let Some(next) = chars.next() {
                match next {
                    '"' => {
                        closed = true;
                        break;
                    }
                    '\\' => text.push(match chars.next() {
                        Some('n') => '\n',
                        Some('r') => '\r',
                        Some('t') => '\t',
                        Some('"') => '"',
                        Some('\\') => '\\',
                        _ => return Err("unsupported string escape".to_owned()),
                    }),
                    _ => text.push(next),
                }
            }
            if !closed {
                return Err("unterminated description string".to_owned());
            }
            tokens.push(Token::Text(text));
        } else if character == ';' {
            tokens.push(Token::Word(";".to_owned()));
        } else {
            let mut word = String::from(character);
            while chars
                .peek()
                .is_some_and(|c| !c.is_whitespace() && !matches!(c, ';' | '"' | '#'))
            {
                word.push(chars.next().expect("peeked character"));
            }
            tokens.push(Token::Word(word));
        }
        if tokens.len() > MAX_TOKENS {
            return Err("source exceeds token limit".to_owned());
        }
    }
    Ok(tokens)
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.'))
}

fn reserved(name: &str) -> bool {
    matches!(
        name,
        "+" | "-"
            | "*"
            | "/"
            | "div"
            | "mod"
            | "dup"
            | "drop"
            | "swap"
            | "over"
            | "rot"
            | "nip"
            | "tuck"
            | "clear"
            | "depth"
            | "pick"
            | "roll"
            | "pi"
            | "e"
            | "exp"
            | "ln"
            | "log2"
            | "log10"
            | "sin"
            | "cos"
            | "tan"
            | "rad"
            | "deg"
            | "pow"
            | "root"
            | "gcd"
            | "lcm"
            | "sqrt"
            | "fact"
            | "approx"
            | "clearmem"
            | "if"
            | "else"
            | "then"
            | "times"
            | "end"
            | "while"
            | "do"
            | "not"
            | "abs"
            | "neg"
    )
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn take_word(&mut self) -> Result<String, String> {
        let token = self
            .tokens
            .get(self.position)
            .ok_or("unexpected end of source")?;
        if let Token::Word(word) = token {
            self.position += 1;
            Ok(word.clone())
        } else {
            Err("expected a word".to_owned())
        }
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        let actual = self.take_word()?;
        if actual == expected {
            Ok(())
        } else {
            Err(format!("expected '{expected}', found '{actual}'"))
        }
    }

    fn block(
        &mut self,
        stops: &[&str],
        depth: usize,
        definitions: bool,
    ) -> Result<Program, String> {
        if depth > MAX_CALL_DEPTH {
            return Err("parser nesting limit exceeded".to_owned());
        }
        let mut code = Vec::new();
        while self.position < self.tokens.len() {
            if stops.contains(&self.tokens[self.position].word()) {
                break;
            }
            let start = self.position;
            let word = self.take_word()?;
            match word.as_str() {
                ":" => {
                    if !definitions {
                        return Err("definitions must be at top level".to_owned());
                    }
                    let name = self.take_word()?;
                    if !valid_name(&name) || reserved(&name) {
                        return Err(format!("invalid or reserved macro name '{name}'"));
                    }
                    let description =
                        if let Some(Token::Text(text)) = self.tokens.get(self.position) {
                            let description = text.clone();
                            self.position += 1;
                            description
                        } else {
                            String::new()
                        };
                    let program = self.block(&[";"], depth + 1, false)?;
                    self.expect(";")?;
                    let source = self.tokens[start..self.position]
                        .iter()
                        .map(Token::source)
                        .collect::<Vec<_>>()
                        .join(" ");
                    code.push(Instruction::Define(
                        name,
                        Arc::new(Definition {
                            description,
                            source,
                            program,
                        }),
                    ));
                }
                "if" => {
                    let branch = code.len();
                    code.push(Instruction::JumpUnless(0));
                    append(&mut code, self.block(&["else", "then"], depth + 1, false)?);
                    let stop = self.take_word()?;
                    if stop == "else" {
                        let jump = code.len();
                        code.push(Instruction::Jump(0));
                        code[branch] = Instruction::JumpUnless(code.len());
                        append(&mut code, self.block(&["then"], depth + 1, false)?);
                        self.expect("then")?;
                        code[jump] = Instruction::Jump(code.len());
                    } else if stop == "then" {
                        code[branch] = Instruction::JumpUnless(code.len());
                    } else {
                        return Err("missing then".to_owned());
                    }
                }
                "times" => {
                    let enter = code.len();
                    code.push(Instruction::Repeat(0));
                    append(&mut code, self.block(&["end"], depth + 1, false)?);
                    self.expect("end")?;
                    code.push(Instruction::Next(enter + 1));
                    code[enter] = Instruction::Repeat(code.len());
                }
                "while" => {
                    let start = code.len();
                    append(&mut code, self.block(&["do"], depth + 1, false)?);
                    self.expect("do")?;
                    let branch = code.len();
                    code.push(Instruction::JumpUnless(0));
                    append(&mut code, self.block(&["end"], depth + 1, false)?);
                    self.expect("end")?;
                    code.push(Instruction::Jump(start));
                    code[branch] = Instruction::JumpUnless(code.len());
                }
                ";" | "else" | "then" | "end" | "do" => return Err(format!("unexpected '{word}'")),
                _ if word.starts_with("let:") || word.starts_with("get:") => {
                    let (operation, name) = word.split_once(':').expect("prefix includes colon");
                    if !valid_name(name) {
                        return Err("invalid local variable name".to_owned());
                    }
                    code.push(if operation == "let" {
                        Instruction::Set(name.to_owned())
                    } else {
                        Instruction::Get(name.to_owned())
                    });
                }
                _ => {
                    for token in crate::engine::tokenize(&word) {
                        code.push(if let Some(value) = parse_literal(token) {
                            Instruction::Push(value)
                        } else {
                            Instruction::Word(token.to_owned())
                        });
                    }
                }
            }
        }
        Ok(Program(code))
    }
}

// Relocate block-local jump targets when embedding code inside another block.
fn append(destination: &mut Vec<Instruction>, program: Program) {
    let offset = destination.len();
    destination.extend(program.0.into_iter().map(|instruction| match instruction {
        Instruction::Jump(to) => Instruction::Jump(to + offset),
        Instruction::JumpUnless(to) => Instruction::JumpUnless(to + offset),
        Instruction::Repeat(to) => Instruction::Repeat(to + offset),
        Instruction::Next(to) => Instruction::Next(to + offset),
        other => other,
    }));
}

pub fn compile(source: &str) -> Result<Program, String> {
    let mut parser = Parser {
        tokens: lex(source)?,
        position: 0,
    };
    parser.block(&[], 0, true)
}

pub fn definitions_only(program: &Program) -> bool {
    program
        .0
        .iter()
        .all(|instruction| matches!(instruction, Instruction::Define(_, _)))
}

pub fn needs_vm(source: &str) -> bool {
    source.contains(['#', ';', '"'])
        || source.split_whitespace().any(|word| {
            matches!(word, ":" | "if" | "while" | "times")
                || word.starts_with("let:")
                || word.starts_with("get:")
        })
}

pub fn execute(
    calc: &mut Calculator,
    program: &Program,
    traced: bool,
) -> Result<Vec<String>, String> {
    let mut vm = Vm {
        steps: MAX_STEPS,
        trace: Vec::new(),
        traced,
    };
    vm.run(calc, program, 0)?;
    Ok(vm.trace)
}

struct Vm {
    steps: usize,
    trace: Vec<String>,
    traced: bool,
}

impl Vm {
    fn run(
        &mut self,
        calc: &mut Calculator,
        program: &Program,
        depth: usize,
    ) -> Result<(), String> {
        if depth >= MAX_CALL_DEPTH {
            return Err("macro call-depth limit exceeded".to_owned());
        }
        let mut locals = BTreeMap::new();
        let mut loops: Vec<usize> = Vec::new();
        let mut ip = 0;
        while let Some(instruction) = program.0.get(ip) {
            if self.steps == 0 {
                return Err("macro execution-step limit exceeded".to_owned());
            }
            self.steps -= 1;
            if self.traced && self.trace.len() < 4096 {
                let text = format!(
                    "depth={depth} ip={ip} {instruction:?} stack={}",
                    calc.depth()
                );
                self.trace.push(text.chars().take(240).collect());
            }
            match instruction {
                Instruction::Push(value) => calc.push_value(value.clone())?,
                Instruction::Word(word) => {
                    if let Some(definition) = calc.macros().get(word).cloned() {
                        self.run(calc, &definition.program, depth + 1)?;
                    } else {
                        calc.evaluate_plain(word)
                            .map_err(|error| error.to_string())?;
                    }
                }
                Instruction::Define(name, definition) => {
                    if calc.macros().len() >= MAX_DEFINITIONS && !calc.macros().contains_key(name) {
                        return Err("macro definition limit exceeded".to_owned());
                    }
                    calc.define_macro(name.clone(), definition.clone());
                }
                Instruction::Jump(target) => {
                    ip = *target;
                    continue;
                }
                Instruction::JumpUnless(target) => {
                    if calc.pop_value("if/while")? == 0 {
                        ip = *target;
                        continue;
                    }
                }
                Instruction::Repeat(target) => {
                    let count = usize::try_from(&calc.pop_value("times")?)
                        .map_err(|_| "times requires a nonnegative integer count")?;
                    if count == 0 {
                        ip = *target;
                        continue;
                    }
                    loops.push(count);
                }
                Instruction::Next(target) => {
                    let remaining = loops.last_mut().ok_or("invalid loop instruction")?;
                    *remaining -= 1;
                    if *remaining > 0 {
                        ip = *target;
                        continue;
                    }
                    loops.pop();
                }
                Instruction::Set(name) => {
                    locals.insert(name.clone(), calc.pop_value("let")?);
                }
                Instruction::Get(name) => {
                    let value = locals
                        .get(name)
                        .ok_or_else(|| format!("unknown local variable '{name}'"))?;
                    calc.push_value(value.clone())?;
                }
            }
            ip += 1;
        }
        Ok(())
    }
}
