#!/usr/bin/env python3
"""Self-contained rpning repair and M2/M3 integration.

Recovers an accidentally overwritten src/main.rs from known Git history,
stages the new modules in the correct paths, applies exact GitHub-based
FIND -> REPLACE anchors and checks only existing TODO boxes.

This requires no separate downloads and no network access.
All changes are preflighted before writing any project files.
"""
from __future__ import annotations

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
pending: dict[Path, tuple[str, str]] = {}


def edit(relative: str, old: str, new: str) -> None:
    path = ROOT / relative
    if path not in pending:
        raw = path.read_bytes()
        newline = "\r\n" if b"\r\n" in raw else "\n"
        pending[path] = (raw.decode("utf-8").replace("\r\n", "\n"), newline)
    content, newline = pending[path]
    occurrences = content.count(old)
    if occurrences != 1:
        raise RuntimeError(f"{relative}: exact FIND matched {occurrences} times (expected 1):\n{old}")
    pending[path] = (content.replace(old, new, 1), newline)



# Plain-text files are embedded so there is no downloads-folder path mix-up.
BUNDLED_FILES = {
    'src/math.rs': '//! Exact mathematical operations for the rational calculator.\n//!\n//! Domain errors never modify the caller\'s stack. Operations are deliberately\n//! exact: an irrational root is rejected rather than silently rounded.\n\nuse malachite::base::num::arithmetic::traits::{CheckedRoot, CheckedSqrt, Gcd, Lcm, UnsignedAbs};\nuse malachite::{Integer, Natural, Rational};\nuse std::fmt;\n\nconst MAX_POWER: u64 = 100_000;\nconst MAX_FACTORIAL: u64 = 10_000;\nconst MAX_ROOT: u64 = 1_024;\n\n#[derive(Debug, Clone, PartialEq, Eq)]\npub enum MathError {\n    IntegerRequired(&\'static str),\n    Range(&\'static str),\n    Domain(&\'static str),\n    IrrationalRoot,\n}\n\nimpl fmt::Display for MathError {\n    fn fmt(&self, f: &mut fmt::Formatter<\'_>) -> fmt::Result {\n        match self {\n            Self::IntegerRequired(op) => write!(f, "\'{op}\' requires integer operands"),\n            Self::Range(reason) => write!(f, "number out of supported range: {reason}"),\n            Self::Domain(reason) => write!(f, "math domain error: {reason}"),\n            Self::IrrationalRoot => write!(f, "root is not rational (no rounding was performed)"),\n        }\n    }\n}\n\nimpl std::error::Error for MathError {}\n\nfn exact_integer(value: &Rational, op: &\'static str) -> Result<Integer, MathError> {\n    Integer::try_from(value.clone()).map_err(|_| MathError::IntegerRequired(op))\n}\n\nfn bounded_integer(value: &Rational, op: &\'static str) -> Result<i64, MathError> {\n    let integer = exact_integer(value, op)?;\n    integer\n        .to_string()\n        .parse::<i64>()\n        .map_err(|_| MathError::Range("integer operand does not fit i64"))\n}\n\n/// Evaluate one exact unary mathematical operation.\npub fn unary(op: &str, x: &Rational) -> Result<Rational, MathError> {\n    match op {\n        "sqrt" => {\n            if x < &Rational::from(0) {\n                return Err(MathError::Domain("square root of a negative value"));\n            }\n            x.clone().checked_sqrt().ok_or(MathError::IrrationalRoot)\n        }\n        "fact" => {\n            let count = bounded_integer(x, "fact")?;\n            if count < 0 {\n                return Err(MathError::Domain("factorial of a negative value"));\n            }\n            let n = count as u64;\n            if n > MAX_FACTORIAL {\n                return Err(MathError::Range("factorial exceeds 10000"));\n            }\n            let mut result = Natural::from(1u8);\n            for i in 2..=n {\n                result *= Natural::from(i);\n            }\n            Ok(Rational::from(result))\n        }\n        _ => unreachable!("the engine dispatches only known unary operations"),\n    }\n}\n\n/// Evaluate one exact binary mathematical operation.\n/// The left operand is Y and the right operand is X.\npub fn binary(op: &str, y: &Rational, x: &Rational) -> Result<Rational, MathError> {\n    match op {\n        "pow" => {\n            let exponent = bounded_integer(x, "pow")?;\n            let magnitude = exponent.unsigned_abs();\n            if magnitude > MAX_POWER {\n                return Err(MathError::Range("absolute exponent exceeds 100000"));\n            }\n            if exponent < 0 && y == &Rational::from(0) {\n                return Err(MathError::Domain("zero to a negative power"));\n            }\n            let mut result = Rational::from(1);\n            let mut factor = y.clone();\n            let mut remaining = magnitude;\n            while remaining > 0 {\n                if remaining & 1 == 1 {\n                    result *= &factor;\n                }\n                remaining >>= 1;\n                if remaining != 0 {\n                    factor = &factor * &factor;\n                }\n            }\n            if exponent < 0 {\n                Ok(Rational::from(1) / result)\n            } else {\n                Ok(result)\n            }\n        }\n        "root" => {\n            let degree = bounded_integer(x, "root")?;\n            if degree <= 0 || degree as u64 > MAX_ROOT {\n                return Err(MathError::Range("root degree must be 1..=1024"));\n            }\n            if degree % 2 == 0 && y < &Rational::from(0) {\n                return Err(MathError::Domain("even root of a negative value"));\n            }\n            y.clone().checked_root(degree).ok_or(MathError::IrrationalRoot)\n        }\n        "gcd" | "lcm" => {\n            let a = exact_integer(y, if op == "gcd" { "gcd" } else { "lcm" })?;\n            let b = exact_integer(x, if op == "gcd" { "gcd" } else { "lcm" })?;\n            let a = a.unsigned_abs();\n            let b = b.unsigned_abs();\n            let result = if op == "gcd" { a.gcd(b) } else { a.lcm(b) };\n            Ok(Rational::from(result))\n        }\n        _ => unreachable!("the engine dispatches only known binary operations"),\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::{MathError, binary, unary};\n    use malachite::Rational;\n\n    fn rational(value: &str) -> Rational {\n        value.parse().expect("valid rational test input")\n    }\n\n    #[test]\n    fn exact_powers_and_inverse_powers() {\n        let cases = [\n            ("2", "10", "1024"),\n            ("2", "-3", "1/8"),\n            ("-2", "3", "-8"),\n            ("0", "0", "1"),\n            ("3/2", "3", "27/8"),\n        ];\n        for (base, exponent, expected) in cases {\n            assert_eq!(binary("pow", &rational(base), &rational(exponent)).unwrap(), rational(expected));\n        }\n        assert!(matches!(binary("pow", &rational("0"), &rational("-1")), Err(MathError::Domain(_))));\n        assert!(matches!(binary("pow", &rational("2"), &rational("1/2")), Err(MathError::IntegerRequired(_))));\n    }\n\n    #[test]\n    fn roots_are_exact_or_explicitly_rejected() {\n        assert_eq!(unary("sqrt", &rational("81/16")).unwrap(), rational("9/4"));\n        assert_eq!(binary("root", &rational("-27/8"), &rational("3")).unwrap(), rational("-3/2"));\n        assert_eq!(binary("root", &rational("16"), &rational("4")).unwrap(), rational("2"));\n        assert!(matches!(unary("sqrt", &rational("2")), Err(MathError::IrrationalRoot)));\n        assert!(matches!(binary("root", &rational("-16"), &rational("2")), Err(MathError::Domain(_))));\n    }\n\n    #[test]\n    fn factorial_and_number_theory() {\n        assert_eq!(unary("fact", &rational("0")).unwrap(), rational("1"));\n        assert_eq!(unary("fact", &rational("5")).unwrap(), rational("120"));\n        assert_eq!(binary("gcd", &rational("-48"), &rational("18")).unwrap(), rational("6"));\n        assert_eq!(binary("lcm", &rational("-12"), &rational("18")).unwrap(), rational("36"));\n        assert_eq!(binary("gcd", &rational("0"), &rational("0")).unwrap(), rational("0"));\n        assert!(matches!(unary("fact", &rational("-1")), Err(MathError::Domain(_))));\n        assert!(matches!(binary("gcd", &rational("1/2"), &rational("3")), Err(MathError::IntegerRequired(_))));\n    }\n}\n',
    'src/precision.rs': '//! Explicit, correctly rounded binary approximation of exact rational values.\n//!\n//! All intermediate calculations use Malachite integers and rationals. No\n//! f32/f64 conversion occurs. The output is a dyadic rational carrying the\n//! precise value of the selected finite-precision binary result.\n\nuse malachite::base::num::arithmetic::traits::{DivRem, FloorLogBasePowerOf2};\nuse malachite::{Natural, Rational};\n\npub const MIN_BITS: u64 = 2;\npub const MAX_BITS: u64 = 4096;\nconst MAX_SHIFT: u64 = 1_000_000;\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]\npub enum Rounding {\n    #[default]\n    NearestEven,\n    TowardZero,\n    Up,\n    Down,\n    AwayFromZero,\n}\n\nimpl Rounding {\n    pub fn parse(input: &str) -> Option<Self> {\n        match input {\n            "nearest" => Some(Self::NearestEven),\n            "zero" => Some(Self::TowardZero),\n            "up" => Some(Self::Up),\n            "down" => Some(Self::Down),\n            "away" => Some(Self::AwayFromZero),\n            _ => None,\n        }\n    }\n\n    pub fn name(self) -> &\'static str {\n        match self {\n            Self::NearestEven => "nearest",\n            Self::TowardZero => "zero",\n            Self::Up => "up",\n            Self::Down => "down",\n            Self::AwayFromZero => "away",\n        }\n    }\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct NumericSettings {\n    pub binary_mode: bool,\n    pub bits: u64,\n    pub rounding: Rounding,\n    pub scientific_display: bool,\n}\n\nimpl Default for NumericSettings {\n    fn default() -> Self {\n        Self {\n            binary_mode: false,\n            bits: 53,\n            rounding: Rounding::NearestEven,\n            scientific_display: false,\n        }\n    }\n}\n\n/// Exactly round a nonzero rational to the nearest representable binary number\n/// with the chosen count of significant bits and unlimited exponent range.\n///\n/// Returns an error if requested precision or intermediate shifts are too\n/// large. The mode uses nearest-even ties, not nearest-away.\npub fn round_binary(value: &Rational, bits: u64, mode: Rounding) -> Result<Rational, &\'static str> {\n    if !(MIN_BITS..=MAX_BITS).contains(&bits) {\n        return Err("binary precision must be between 2 and 4096 bits");\n    }\n    if value == &Rational::from(0) {\n        return Ok(Rational::from(0));\n    }\n\n    let negative = value < &Rational::from(0);\n    let positive = if negative { -value.clone() } else { value.clone() };\n    let exponent = (&positive).floor_log_base_power_of_2(1);\n    let shift = i64::try_from(bits - 1)\n        .map_err(|_| "precision is too large")?\n        .checked_sub(exponent)\n        .ok_or("binary exponent is out of range")?;\n    let shift_magnitude = shift.unsigned_abs();\n    if shift_magnitude > MAX_SHIFT {\n        return Err("binary exponent shift exceeds the safety limit");\n    }\n\n    let mut numerator = positive.numerator_ref().clone();\n    let mut denominator = positive.denominator_ref().clone();\n    if shift >= 0 {\n        numerator <<= shift_magnitude;\n    } else {\n        denominator <<= shift_magnitude;\n    }\n    let (mut quotient, remainder) = numerator.div_rem(denominator.clone());\n    let rem_nonzero = remainder != 0;\n    let round_up = match mode {\n        Rounding::TowardZero => false,\n        Rounding::AwayFromZero => rem_nonzero,\n        Rounding::Up => !negative && rem_nonzero,\n        Rounding::Down => negative && rem_nonzero,\n        Rounding::NearestEven => {\n            let twice = &remainder + &remainder;\n            twice > denominator\n                || (twice == denominator && (&quotient % Natural::from(2u8)) != 0)\n        }\n    };\n    if round_up {\n        quotient += Natural::from(1u8);\n    }\n\n    let output = if shift >= 0 {\n        Rational::from(quotient) / Rational::from(Natural::from(1u8) << shift_magnitude)\n    } else {\n        Rational::from(quotient << shift_magnitude)\n    };\n    Ok(if negative { -output } else { output })\n}\n\n#[cfg(test)]\nmod tests {\n    use super::{Rounding, round_binary};\n    use malachite::Rational;\n\n    fn r(s: &str) -> Rational {\n        s.parse().unwrap()\n    }\n\n    #[test]\n    fn rounding_is_exact_and_configurable() {\n        assert_eq!(round_binary(&r("1/3"), 4, Rounding::NearestEven).unwrap(), r("11/32"));\n        assert_eq!(round_binary(&r("1/3"), 4, Rounding::TowardZero).unwrap(), r("5/16"));\n        assert_eq!(round_binary(&r("1/3"), 4, Rounding::Up).unwrap(), r("11/32"));\n        assert_eq!(round_binary(&r("-1/3"), 4, Rounding::Down).unwrap(), r("-11/32"));\n        assert_eq!(round_binary(&r("-1/3"), 4, Rounding::Up).unwrap(), r("-5/16"));\n    }\n\n    #[test]\n    fn ties_are_even_including_negative_values() {\n        assert_eq!(round_binary(&r("5/4"), 2, Rounding::NearestEven).unwrap(), r("1"));\n        assert_eq!(round_binary(&r("7/4"), 2, Rounding::NearestEven).unwrap(), r("2"));\n        assert_eq!(round_binary(&r("-7/4"), 2, Rounding::NearestEven).unwrap(), r("-2"));\n    }\n\n    #[test]\n    fn exact_values_survive_approximation() {\n        for input in ["0", "1", "-1", "3/2", "1024", "1/256"] {\n            let value = r(input);\n            assert_eq!(round_binary(&value, 53, Rounding::NearestEven).unwrap(), value);\n        }\n    }\n\n    #[test]\n    fn invalid_precision_is_rejected() {\n        assert!(round_binary(&r("1"), 1, Rounding::NearestEven).is_err());\n        assert!(round_binary(&r("1"), 4097, Rounding::NearestEven).is_err());\n    }\n}\n',
    'src/scientific.rs': '//! Lossless base-10 scientific display of terminating rational values.\n//!\n//! Recurring rationals remain exact fractions. We never manufacture a\n//! finite decimal approximation merely to make the output look scientific.\n\nuse crate::number::{Number, format_number};\n\npub fn format_scientific(value: &Number) -> String {\n    let decimal = format_number(value);\n    if decimal.contains(\'/\') || decimal == "0" {\n        return decimal;\n    }\n    let (negative, unsigned) = match decimal.strip_prefix(\'-\') {\n        Some(rest) => ("-", rest),\n        None => ("", decimal.as_str()),\n    };\n    let before_dot = unsigned.split(\'.\').next().unwrap_or(unsigned);\n    let digits_before_dot = before_dot.len();\n    let all_digits: String = unsigned.chars().filter(|ch| *ch != \'.\').collect();\n    let Some(first_nonzero) = all_digits.bytes().position(|byte| byte != b\'0\') else {\n        return "0".to_owned();\n    };\n    let exponent = digits_before_dot as i64 - first_nonzero as i64 - 1;\n    let mut significand = all_digits[first_nonzero..].trim_end_matches(\'0\').to_owned();\n    if significand.len() > 1 {\n        significand.insert(1, \'.\');\n    }\n    if exponent == 0 {\n        format!("{negative}{significand}")\n    } else {\n        format!("{negative}{significand}e{exponent}")\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::format_scientific;\n    use crate::number::parse_literal;\n\n    #[test]\n    fn formats_terminating_numbers_losslessly() {\n        let cases = [\n            ("0", "0"),\n            ("1", "1"),\n            ("12345", "1.2345e4"),\n            ("-500", "-5e2"),\n            ("0.00125", "1.25e-3"),\n            ("-0.5", "-5e-1"),\n            ("10/4", "2.5"),\n            ("1/3", "1/3"),\n        ];\n        for (input, expected) in cases {\n            let value = parse_literal(input).unwrap();\n            let output = format_scientific(&value);\n            assert_eq!(output, expected);\n            assert_eq!(parse_literal(&output), Some(value));\n        }\n    }\n}\n',
    'src/storage.rs': '//! Versioned, lossless persistence for exact named registers.\n//!\n//! Only explicit :save and :load use this module. The format is plain UTF-8:\n//! a version header followed by name, tab, canonical Rational value per line.\n\nuse std::collections::BTreeMap;\nuse std::env;\nuse std::fs::{self, File};\nuse std::io::{self, Write};\nuse std::path::{Path, PathBuf};\n\nuse crate::number::{Number, parse_literal};\n\nconst HEADER: &str = "RPNING-REGISTERS-1";\nconst MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;\nconst MAX_REGISTERS: usize = 10_000;\n\npub fn default_register_path() -> PathBuf {\n    #[cfg(windows)]\n    {\n        if let Some(path) = env::var_os("APPDATA") {\n            return PathBuf::from(path).join("rpning").join("registers.txt");\n        }\n    }\n    if let Some(path) = env::var_os("XDG_DATA_HOME") {\n        return PathBuf::from(path).join("rpning").join("registers.txt");\n    }\n    if let Some(path) = env::var_os("HOME") {\n        return PathBuf::from(path)\n            .join(".local")\n            .join("share")\n            .join("rpning")\n            .join("registers.txt");\n    }\n    PathBuf::from("rpning-registers.txt")\n}\n\npub fn load(path: &Path) -> io::Result<BTreeMap<String, Number>> {\n    let size = fs::metadata(path)?.len();\n    if size > MAX_FILE_BYTES {\n        return Err(invalid("register file exceeds 16 MiB"));\n    }\n    let contents = fs::read_to_string(path)?;\n    let mut lines = contents.lines();\n    if lines.next() != Some(HEADER) {\n        return Err(invalid("unsupported register file version"));\n    }\n    let mut registers = BTreeMap::new();\n    for line in lines {\n        let (name, value_text) = line\n            .split_once(\'\\t\')\n            .ok_or_else(|| invalid("malformed register record"))?;\n        if !valid_name(name) {\n            return Err(invalid("invalid register name"));\n        }\n        let value = parse_literal(value_text)\n            .ok_or_else(|| invalid("invalid rational register value"))?;\n        if registers.insert(name.to_owned(), value).is_some() {\n            return Err(invalid("duplicate register name"));\n        }\n        if registers.len() > MAX_REGISTERS {\n            return Err(invalid("too many registers"));\n        }\n    }\n    Ok(registers)\n}\n\n/// Write a temporary file in the same directory, sync it, then rename it.\n/// A failed write leaves the original destination unchanged.\npub fn save<\'a>(path: &Path, registers: impl Iterator<Item = (&\'a str, &\'a Number)>) -> io::Result<()> {\n    if let Some(parent) = path.parent() {\n        if !parent.as_os_str().is_empty() {\n            fs::create_dir_all(parent)?;\n        }\n    }\n    let mut data = String::from(HEADER);\n    data.push(\'\\n\');\n    let mut count = 0;\n    for (name, value) in registers {\n        if !valid_name(name) {\n            return Err(invalid("invalid register name"));\n        }\n        count += 1;\n        if count > MAX_REGISTERS {\n            return Err(invalid("too many registers"));\n        }\n        data.push_str(name);\n        data.push(\'\\t\');\n        data.push_str(&value.to_string());\n        data.push(\'\\n\');\n        if data.len() as u64 > MAX_FILE_BYTES {\n            return Err(invalid("register file exceeds 16 MiB"));\n        }\n    }\n    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));\n    let write_result = (|| {\n        let mut file = File::create(&temporary)?;\n        file.write_all(data.as_bytes())?;\n        file.sync_all()?;\n        fs::rename(&temporary, path)\n    })();\n    if write_result.is_err() {\n        let _ = fs::remove_file(&temporary);\n    }\n    write_result\n}\n\nfn valid_name(name: &str) -> bool {\n    name.len() <= 64\n        && name\n            .bytes()\n            .next()\n            .is_some_and(|b| b.is_ascii_alphabetic() || b == b\'_\')\n        && name\n            .bytes()\n            .skip(1)\n            .all(|b| b.is_ascii_alphanumeric() || b == b\'_\')\n}\n\nfn invalid(reason: &\'static str) -> io::Error {\n    io::Error::new(io::ErrorKind::InvalidData, reason)\n}\n\n#[cfg(test)]\nmod tests {\n    use super::{load, save};\n    use malachite::Rational;\n    use std::collections::BTreeMap;\n    use std::fs;\n    use std::sync::atomic::{AtomicU64, Ordering};\n\n    static COUNTER: AtomicU64 = AtomicU64::new(0);\n\n    fn unique_file() -> std::path::PathBuf {\n        let number = COUNTER.fetch_add(1, Ordering::Relaxed);\n        std::env::temp_dir().join(format!("rpning-register-test-{}-{number}.txt", std::process::id()))\n    }\n\n    #[test]\n    fn exact_values_survive_serialization() {\n        let path = unique_file();\n        let data = BTreeMap::from([\n            ("huge".to_owned(), "123456789012345678901234567890".parse::<Rational>().unwrap()),\n            ("third".to_owned(), "-1/3".parse::<Rational>().unwrap()),\n        ]);\n        save(&path, data.iter().map(|(k, v)| (k.as_str(), v))).unwrap();\n        let loaded = load(&path).unwrap();\n        assert_eq!(loaded, data);\n        fs::remove_file(path).unwrap();\n    }\n\n    #[test]\n    fn invalid_files_fail_without_partial_success() {\n        let path = unique_file();\n        fs::write(&path, "RPNING-REGISTERS-1\\nvalid\\t1/3\\nbad.name\\t2\\n").unwrap();\n        assert!(load(&path).is_err());\n        fs::remove_file(path).unwrap();\n    }\n}\n',
    'tests/math_cli.rs': 'use std::io::Write;\nuse std::process::{Command, Stdio};\n\nfn run_script(input: &str) -> (bool, String, String) {\n    let mut child = Command::new(env!("CARGO_BIN_EXE_rpn"))\n        .stdin(Stdio::piped())\n        .stdout(Stdio::piped())\n        .stderr(Stdio::piped())\n        .spawn()\n        .expect("failed to launch rpn");\n    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();\n    let output = child.wait_with_output().unwrap();\n    (\n        output.status.success(),\n        String::from_utf8(output.stdout).unwrap(),\n        String::from_utf8(output.stderr).unwrap(),\n    )\n}\n\n#[test]\nfn exact_math_and_scientific_formatting() {\n    let (ok, stdout, stderr) = run_script(\n        "2 10 pow\\nclear 81/16 sqrt\\nclear -27 3 root\\nclear 6 fact\\nclear -48 18 gcd\\nclear 12 18 lcm\\n:display scientific\\nclear 12345\\n",\n    );\n    assert!(ok, "{stderr}");\n    assert_eq!(\n        stdout.lines().collect::<Vec<_>>(),\n        ["1024", "2.25", "-3", "720", "6", "36", "display: scientific", "1.2345e4"]\n    );\n}\n\n#[test]\nfn binary_approximation_and_display_settings() {\n    let (ok, stdout, stderr) = run_script(\n        ":mode binary\\n:precision 4\\n1 3 /\\n:round zero\\nclear 1 3 /\\n:mode exact\\nclear 1 3 /\\napprox\\n:settings\\n",\n    );\n    assert!(ok, "{stderr}");\n    assert_eq!(\n        stdout.lines().collect::<Vec<_>>(),\n        [\n            "numeric mode: binary",\n            "precision: 4 bits",\n            "0.34375",\n            "rounding: zero",\n            "0.3125",\n            "numeric mode: exact",\n            "1/3",\n            "0.3125",\n            "mode=exact precision=4 bits rounding=zero display=auto",\n        ]\n    );\n}\n\n#[test]\nfn math_errors_are_atomic_in_interactive_mode() {\n    let (ok, stdout, stderr) = run_script("5\\n2 sqrt\\n:stack\\n:undo\\n:stack\\n");\n    // Interactive errors are intentionally nonfatal, unlike piped stdin.\n    assert!(!ok);\n    assert!(stderr.contains("root is not rational"));\n    assert!(stdout.contains("[0] 5"));\n}\n',
    'tools/precision_oracle.py': '#!/usr/bin/env python3\n"""Independent exact Python Fraction oracle for rpning binary rounding.\n\nUsage on Windows:\n    python tools/precision_oracle.py target/debug/rpn.exe\nOn Linux:\n    python3 tools/precision_oracle.py target/debug/rpn\n\nNo Rust/Rug/Python third-party dependencies are required. Verifies every\nsupported rounding direction, tiny values, huge values and midpoint ties.\n"""\nfrom __future__ import annotations\n\nimport random\nimport subprocess\nimport sys\nfrom fractions import Fraction\nfrom pathlib import Path\n\n\ndef reference(value: Fraction, bits: int, mode: str) -> Fraction:\n    if value == 0:\n        return Fraction(0)\n    negative = value < 0\n    positive = abs(value)\n    numerator, denominator = positive.numerator, positive.denominator\n    exponent = numerator.bit_length() - denominator.bit_length()\n    if exponent >= 0:\n        if numerator < (denominator << exponent):\n            exponent -= 1\n    elif (numerator << -exponent) < denominator:\n        exponent -= 1\n    shift = bits - 1 - exponent\n    if shift >= 0:\n        numerator <<= shift\n    else:\n        denominator <<= -shift\n    quotient, rem = divmod(numerator, denominator)\n    if mode == "nearest":\n        inc = 2 * rem > denominator or (2 * rem == denominator and quotient % 2 != 0)\n    elif mode == "up":\n        inc = bool(rem) and not negative\n    elif mode == "down":\n        inc = bool(rem) and negative\n    elif mode == "away":\n        inc = bool(rem)\n    elif mode == "zero":\n        inc = False\n    else:\n        raise ValueError(mode)\n    quotient += int(inc)\n    result = Fraction(quotient, 1 << shift) if shift >= 0 else Fraction(quotient << -shift)\n    return -result if negative else result\n\n\ndef test_vectors() -> list[Fraction]:\n    samples = [\n        Fraction(0), Fraction(1, 3), Fraction(-1, 3), Fraction(5, 4),\n        Fraction(7, 4), Fraction(-7, 4), Fraction(1, 2**150),\n        Fraction(2**180), Fraction((2**200) + 123, 3),\n        Fraction(-((2**200) + 123), 3), Fraction(1, 10),\n        Fraction(-25, 16), Fraction(2**200, 2**155),\n    ]\n    rng = random.Random(20261118)\n    for _ in range(25):\n        num = rng.getrandbits(rng.randint(1, 120))\n        den = max(rng.getrandbits(rng.randint(1, 120)), 1)\n        samples.append(Fraction(num * (-1 if rng.getrandbits(1) else 1), den))\n    return samples\n\n\ndef main(executable: Path) -> None:\n    if not executable.is_file():\n        raise SystemExit(f"Calculator executable not found: {executable}")\n    cases = test_vectors()\n    passed = 0\n    for bits in [2, 3, 4, 8, 53, 256]:\n        for mode in ["nearest", "zero", "up", "down", "away"]:\n            commands = [":mode binary", f":precision {bits}", f":round {mode}"]\n            commands.extend(f"clear {value.numerator}/{value.denominator}" for value in cases)\n            output = subprocess.run(\n                [str(executable)], input="\\n".join(commands) + "\\n", text=True,\n                capture_output=True, check=False,\n            )\n            if output.returncode:\n                raise SystemExit(f"rpn failed at {bits=} {mode=}: {output.stderr}")\n            lines = output.stdout.splitlines()\n            if len(lines) != len(cases) + 3:\n                raise SystemExit(f"unexpected output count for {bits=} {mode=}: {lines[:8]}")\n            for value, actual_text in zip(cases, lines[3:]):\n                expected = reference(value, bits, mode)\n                actual = Fraction(actual_text)\n                if actual != expected:\n                    raise SystemExit(\n                        f"ROUNDING MISMATCH {bits=} {mode=} {value=} "\n                        f"expected={expected} actual={actual} printed={actual_text}"\n                    )\n                passed += 1\n    print(f"PASS: {passed} independently checked binary-rounding results")\n\n\nif __name__ == "__main__":\n    if len(sys.argv) != 2:\n        raise SystemExit("Usage: python tools/precision_oracle.py path/to/rpn[.exe]")\n    main(Path(sys.argv[1]).resolve())\n',
    'docs/M2-BENCHMARK.md': "# M2 numerical backend — benchmark and decision\n\nSource: user-run `comparison.csv` and `comparison-report.md`, 2026-11-18, WSL2 Linux x86_64. The benchmark ran 400 iterations × 7 repeats per case; reported timings are median nanoseconds per arithmetic operation **including two operand clones**, excluding parsing/input generation/formatting.\n\n## Results\n\n27 size/operation comparisons were available. Malachite won **17**, Rug **10**. By category:\n\n| Category | Malachite wins | Rug wins | Noteworthy result |\n|---|---:|---:|---|\n| Integer | 8 | 4 | At 20 decimal digits, Malachite was 1.80× faster for multiplication and 1.91× for division. At 310 digits, Rug was 1.41× faster at multiplication. |\n| Rational | 6 | 3 | Malachite won the smaller cases; Rug won all three 40-digits-per-part cases by ~1.56–1.59×; the largest cases were close. |\n| Binary float | 3 | 3 | Malachite led 53-bit addition (2.10×), while Rug led 256-bit multiplication (1.53×) and 1024-bit multiplication (1.45×). |\n\nThe geometric mean of per-case Malachite/Rug execution-time ratios was approximately **0.91** (a modest average Malachite advantage across this workload). This is *not* an end-to-end speedup, and the choice of cases/weights matters.\n\n## Decision: Keep Malachite as the runtime backend\n\n- Existing exact integer and rational operations already use Malachite.\n- The Windows-first `rpning` executable remains pure Rust and does not require shipping GMP/MPFR.\n- The measured small-number results favor Malachite, an important interactive-calculator workload.\n- Rug remains a **benchmark/reference dependency only**, in the separately compiled `benchmarks` crate. Its advantage in some larger operations is worth revisiting after profiling real-world use, not enough to justify a backend migration now.\n- Binary approximation is initially implemented by rounding exact rational arithmetic to a configurable number of binary significant bits. This is a **dyadic rational approximation**, not full IEEE 754 or Malachite `Float` semantics. The separate benchmark's float cases do not validate this calculator mode.\n\n## Limitations and follow-ups\n\n1. One WSL2 host and one benchmark configuration; the samples may include scheduling and power-state noise.\n2. The benchmark measures isolated operations and clones, not `Calculator::evaluate`, allocation behavior, undo/redo snapshots, CLI parsing or formatting.\n3. Integer and rational cross-checks were enabled. Floating cases were performance-only and do **not** prove rounding accuracy.\n4. A future M2/M8 run should cover startup, end-to-end stack throughput, parsing/formatting, larger operand distributions and correctness against independent precision references.\n5. Binary approximation uses unlimited exponent range, so NaN, infinity, signed zero, exact MPFR transcendental rounding and special IEEE behavior remain outside the current implementation.\n\n**TODO policy:** The two existing benchmark checkboxes and the backend-selection checkbox may be checked. Do not check M2 or M3 features that are not implemented and verified merely to declare a whole milestone complete.\n",
}

BASELINE_COMMIT = "cf4262affb6c4acc4948e4132b15816a3d3bdba2"

def _git_show(object_name: str) -> str:
    result = subprocess.run(
        ["git", "show", object_name], cwd=ROOT,
        capture_output=True, check=False,
    )
    if result.returncode:
        raise RuntimeError(
            f"git show {object_name} failed; make sure the repository has its "
            "complete local history (git fetch origin main). "
            + result.stderr.decode("utf-8", "replace")
        )
    return result.stdout.decode("utf-8").replace("\r\n", "\n")


def recover_and_stage() -> None:
    cargo = ROOT / "Cargo.toml"
    if not cargo.is_file() or 'name = "rpning"' not in cargo.read_text(encoding="utf-8"):
        raise RuntimeError("run from the rpning repository; Cargo.toml was not recognized")

    main_path = ROOT / "src/main.rs"
    existing_raw = main_path.read_bytes()
    newline = "\r\n" if b"\r\n" in existing_raw else "\n"
    current = existing_raw.decode("utf-8").replace("\r\n", "\n")
    misplaced_math = (
        current.startswith("//! Exact mathematical operations for the rational calculator.")
        and "pub fn binary(" in current
        and "fn main()" not in current
    )

    if misplaced_math:
        tracked = _git_show("HEAD:src/main.rs")
        if current != tracked:
            math_path = ROOT / "src/math.rs"
            if math_path.exists():
                raise RuntimeError(
                    "src/main.rs contains locally modified math code and src/math.rs "
                    "already exists. No files changed; reconcile these before recovery."
                )
            print("note: local math formatting differs from HEAD; preserving it in src/math.rs")
        original = _git_show(f"{BASELINE_COMMIT}:src/main.rs")
        if "fn main()" not in original or "use rpning::Calculator;" not in original:
            raise RuntimeError("the historical CLI could not be verified")
        pending[main_path] = (original, newline)
        print(f"recovery staged: src/main.rs <- Git commit {BASELINE_COMMIT[:12]}")
    elif "fn main()" not in current or "use rpning::Calculator;" not in current:
        raise RuntimeError(
            "src/main.rs has an unfamiliar layout. No files changed; "
            "the repair refuses to overwrite it."
        )

    for relative, signature in {
        "src/math.rs": "//! Exact mathematical operations for the rational calculator.",
        "src/precision.rs": "//! Explicit, correctly rounded binary approximation",
        "src/scientific.rs": "//! Lossless base-10 scientific display",
        "src/storage.rs": "//! Versioned, lossless persistence",
        "tests/math_cli.rs": "fn run_script(input: &str)",
        "tools/precision_oracle.py": "Independent exact Python Fraction oracle",
        "docs/M2-BENCHMARK.md": "rpning",
    }.items():
        path = ROOT / relative
        if path.is_file():
            data = path.read_text(encoding="utf-8")
            if signature not in data:
                raise RuntimeError(
                    f"{relative} exists but is not the expected module. "
                    "No files changed; check your downloaded file placement."
                )
            print(f"already present: {relative}")
            continue
        contents = (
            current if relative == "src/math.rs" and misplaced_math
            else BUNDLED_FILES[relative]
        )
        pending[path] = (contents.replace("\r\n", "\n"), "\n")
        print(f"new file staged: {relative}")

def main() -> None:
    # Safe to rerun after a successful install; partial installs still fail.
    lib = (ROOT / "src/lib.rs").read_text(encoding="utf-8")
    engine = (ROOT / "src/engine.rs").read_text(encoding="utf-8")
    cli = (ROOT / "src/main.rs").read_text(encoding="utf-8")
    if ("pub mod precision;" in lib and "pub mod math;" in lib
            and "fn quantize(&self" in engine and "fn configure(calc:" in cli):
        print("M2/M3 integration is already installed; no files changed.")
        return
    recover_and_stage()

    edit("src/lib.rs", "pub mod engine;\npub mod number;", "pub mod engine;\npub mod math;\npub mod number;\npub mod precision;\npub mod scientific;\npub mod storage;")

    edit("src/engine.rs", "use crate::number::{Number, parse_literal};", "use crate::math;\nuse crate::number::{Number, parse_literal};\nuse crate::precision::{MAX_BITS, MIN_BITS, NumericSettings, Rounding, round_binary};")
    edit("src/engine.rs", "    IntegerOperandsRequired(String),\n}", "    IntegerOperandsRequired(String),\n    Math(String),\n}")
    edit("src/engine.rs", """            Self::IntegerOperandsRequired(operator) => {
                write!(f, "'{operator}' requires integer operands")
            }
""", """            Self::IntegerOperandsRequired(operator) => {
                write!(f, "'{operator}' requires integer operands")
            }
            Self::Math(message) => write!(f, "{message}"),
""")
    edit("src/engine.rs", """pub struct Calculator {
    stack: Vec<Number>,
    registers: BTreeMap<String, Number>,
    undo_history: Vec<Snapshot>,
    redo_history: Vec<Snapshot>,
}""", """pub struct Calculator {
    stack: Vec<Number>,
    registers: BTreeMap<String, Number>,
    settings: NumericSettings,
    undo_history: Vec<Snapshot>,
    redo_history: Vec<Snapshot>,
}""")
    edit("src/engine.rs", """    /// Restore the previous successful calculator state.
""", """    /// Current binary approximation and display settings.
    pub fn settings(&self) -> NumericSettings {
        self.settings
    }

    pub fn set_binary_mode(&mut self, enabled: bool) {
        self.settings.binary_mode = enabled;
    }

    pub fn set_precision_bits(&mut self, bits: u64) -> Result<(), CalcError> {
        if !(MIN_BITS..=MAX_BITS).contains(&bits) {
            return Err(CalcError::Math(format!(
                "precision must be between {MIN_BITS} and {MAX_BITS} bits"
            )));
        }
        self.settings.bits = bits;
        Ok(())
    }

    pub fn set_rounding(&mut self, rounding: Rounding) {
        self.settings.rounding = rounding;
    }

    pub fn set_scientific_display(&mut self, enabled: bool) {
        self.settings.scientific_display = enabled;
    }

    /// Replace persisted registers as one undoable transaction.
    pub fn restore_registers(&mut self, loaded: BTreeMap<String, Number>) -> Result<(), CalcError> {
        for name in loaded.keys() {
            validate_register_name(name)?;
        }
        if loaded != self.registers {
            let previous = Snapshot {
                stack: self.stack.clone(),
                registers: std::mem::replace(&mut self.registers, loaded),
            };
            push_bounded(&mut self.undo_history, previous);
            self.redo_history.clear();
        }
        Ok(())
    }

    fn quantize(&self, value: Number) -> Result<Number, CalcError> {
        if self.settings.binary_mode {
            round_binary(&value, self.settings.bits, self.settings.rounding)
                .map_err(|message| CalcError::Math(message.to_owned()))
        } else {
            Ok(value)
        }
    }

    /// Restore the previous successful calculator state.
""")
    edit("src/engine.rs", """                    let x = working.remove(len - 1);
                    let y = working.remove(len - 2);

                    if token == "/" && x == 0 {""", """                    let x = self.quantize(working.remove(len - 1))?;
                    let y = self.quantize(working.remove(len - 2))?;

                    if token == "/" && x == 0 {""")
    edit("src/engine.rs", """                    working.push(result);
                }

                "div" | "mod" => {""", """                    working.push(self.quantize(result)?);
                }

                "div" | "mod" => {""")
    edit("src/engine.rs", """                _ if token.starts_with("sto:") => {""", """                "pow" | "root" | "gcd" | "lcm" => {
                    require_depth(&working, 2, token)?;
                    let len = working.len();
                    let result = math::binary(token, &working[len - 2], &working[len - 1])
                        .map_err(|error| CalcError::Math(error.to_string()))?;
                    working.truncate(len - 2);
                    working.push(self.quantize(result)?);
                }

                "sqrt" | "fact" => {
                    require_depth(&working, 1, token)?;
                    let len = working.len();
                    let result = math::unary(token, &working[len - 1])
                        .map_err(|error| CalcError::Math(error.to_string()))?;
                    working.pop();
                    working.push(self.quantize(result)?);
                }

                "approx" => {
                    require_depth(&working, 1, token)?;
                    let len = working.len();
                    working[len - 1] = round_binary(
                        &working[len - 1], self.settings.bits, self.settings.rounding
                    ).map_err(|message| CalcError::Math(message.to_owned()))?;
                }

                _ if token.starts_with("sto:") => {""")
    edit("src/engine.rs", """                    working.push(value);
                }

                _ if token.starts_with("del:") => {""", """                    working.push(self.quantize(value)?);
                }

                _ if token.starts_with("del:") => {""")
    edit("src/engine.rs", """                    working.push(value);
                }
            }
        }

        if working != self.stack || working_registers != self.registers {""", """                    working.push(self.quantize(value)?);
                }
            }
        }

        if working != self.stack || working_registers != self.registers {""")
    edit("src/engine.rs", """    #[test]
    fn failed_lines_do_not_mutate_the_stack() {""", """    #[test]
    fn exact_advanced_math_preserves_atomicity_and_history() {
        let mut calc = Calculator::new();
        calc.evaluate("2 10 pow").unwrap();
        assert_stack(&calc, &["1024"]);
        calc.evaluate("9/4 sqrt").unwrap();
        assert_stack(&calc, &["1024", "3/2"]);
        let before = calc.stack().to_vec();
        assert!(calc.evaluate("2 sqrt").is_err());
        assert_eq!(calc.stack(), before);
        assert!(calc.undo());
        assert_stack(&calc, &["1024"]);
        calc.evaluate("clear 48 18 gcd 5 fact +").unwrap();
        assert_stack(&calc, &["126"]);
    }

    #[test]
    fn binary_mode_rounds_results_without_f64() {
        let mut calc = Calculator::new();
        calc.set_precision_bits(4).unwrap();
        calc.set_binary_mode(true);
        calc.evaluate("1 3 /").unwrap();
        assert_stack(&calc, &["11/32"]);
        calc.set_rounding(crate::precision::Rounding::TowardZero);
        calc.evaluate("clear 1 3 /").unwrap();
        assert_stack(&calc, &["5/16"]);
        calc.set_binary_mode(false);
        calc.evaluate("clear 1 3 /").unwrap();
        assert_stack(&calc, &["1/3"]);
    }

    #[test]
    fn restore_registers_is_undoable() {
        let mut calc = Calculator::new();
        calc.evaluate("5 sto:old").unwrap();
        let loaded = std::collections::BTreeMap::from([
            ("new".to_owned(), crate::number::parse_literal("1/3").unwrap()),
        ]);
        calc.restore_registers(loaded).unwrap();
        assert!(calc.register("old").is_none());
        assert_eq!(calc.register("new").unwrap().to_string(), "1/3");
        assert!(calc.undo());
        assert_eq!(calc.register("old").unwrap().to_string(), "5");
    }

    #[test]
    fn failed_lines_do_not_mutate_the_stack() {""")

    edit("src/main.rs", """use rpning::Calculator;
use rpning::number::format_number;""", """use rpning::Calculator;
use rpning::number::{Number, format_number};
use rpning::precision::Rounding;
use rpning::scientific::format_scientific;
use rpning::storage;""")
    edit("src/main.rs", """    if let Some(top) = calc.top() {
        println!("{}", format_number(top));
    }

    Ok(())
}""", """    if let Some(top) = calc.top() {
        println!("{}", format_for_calc(&calc, top));
    }

    Ok(())
}""")
    edit("src/main.rs", """                        println!("[{index}] {}", format_number(value));""", """                        println!("[{index}] {}", format_for_calc(&calc, value));""")
    edit("src/main.rs", """                        println!("{name} = {}", format_number(value));""", """                        println!("{name} = {}", format_for_calc(&calc, value));""")
    edit("src/main.rs", """                if changed {
                    if let Some(top) = calc.top() {
                        println!("{}", format_number(top));""", """                if changed {
                    if let Some(top) = calc.top() {
                        println!("{}", format_for_calc(&calc, top));""")
    edit("src/main.rs", """            expression => match calc.evaluate(expression) {
                Ok(()) => {
                    if let Some(top) = calc.top() {
                        println!("{}", format_number(top));""", """            expression => match calc.evaluate(expression) {
                Ok(()) => {
                    if let Some(top) = calc.top() {
                        println!("{}", format_for_calc(&calc, top));""")
    edit("src/main.rs", """            command @ (":undo" | ":redo") => {""", """            ":save" => {
                let path = storage::default_register_path();
                match storage::save(&path, calc.registers()) {
                    Ok(()) => println!("saved registers: {}", path.display()),
                    Err(error) => {
                        eprintln!("error: could not save registers: {error}");
                        if !interactive {
                            had_error = true;
                        }
                    }
                }
            }

            ":load" => {
                let path = storage::default_register_path();
                let result = storage::load(&path)
                    .and_then(|values| calc.restore_registers(values).map_err(io::Error::other));
                match result {
                    Ok(()) => println!("loaded registers: {}", path.display()),
                    Err(error) => {
                        eprintln!("error: could not load registers: {error}");
                        if !interactive {
                            had_error = true;
                        }
                    }
                }
            }

            command if command == ":settings"
                || command.starts_with(":precision ")
                || command.starts_with(":round ")
                || command.starts_with(":mode ")
                || command.starts_with(":display ") =>
            {
                match configure(&mut calc, command) {
                    Ok(message) => println!("{message}"),
                    Err(message) => {
                        eprintln!("error: {message}");
                        if !interactive {
                            had_error = true;
                        }
                    }
                }
            }

            command @ (":undo" | ":redo") => {""")
    edit("src/main.rs", """fn print_help() {""", """fn format_for_calc(calc: &Calculator, value: &Number) -> String {
    if calc.settings().scientific_display {
        format_scientific(value)
    } else {
        format_number(value)
    }
}

fn configure(calc: &mut Calculator, command: &str) -> Result<String, String> {
    let words: Vec<&str> = command.split_whitespace().collect();
    match words.as_slice() {
        [":settings"] => {
            let settings = calc.settings();
            Ok(format!(
                "mode={} precision={} bits rounding={} display={}",
                if settings.binary_mode { "binary" } else { "exact" },
                settings.bits,
                settings.rounding.name(),
                if settings.scientific_display { "scientific" } else { "auto" },
            ))
        }
        [":mode", mode @ ("exact" | "binary")] => {
            calc.set_binary_mode(*mode == "binary");
            Ok(format!("numeric mode: {mode}"))
        }
        [":precision", bits] => {
            let bits = bits.parse::<u64>().map_err(|_| "invalid bit count".to_owned())?;
            calc.set_precision_bits(bits).map_err(|error| error.to_string())?;
            Ok(format!("precision: {bits} bits"))
        }
        [":round", mode] => {
            let rounding = Rounding::parse(mode).ok_or("rounding: nearest|zero|up|down|away")?;
            calc.set_rounding(rounding);
            Ok(format!("rounding: {}", rounding.name()))
        }
        [":display", mode @ ("auto" | "scientific")] => {
            calc.set_scientific_display(*mode == "scientific");
            Ok(format!("display: {mode}"))
        }
        _ => Err("invalid settings command; see :help".to_owned()),
    }
}

fn print_help() {""")
    edit("src/main.rs", """    println!("  div mod                   Integer quotient and remainder");""", """    println!("  div mod                   Integer quotient and remainder");
    println!("  pow sqrt root fact gcd lcm Exact advanced mathematics");
    println!("  approx                    Round X to current binary precision");""")
    edit("src/main.rs", """    println!("  :registers                List named memory registers");""", """    println!("  :registers                List named memory registers");
    println!("  :save / :load             Save/load exact registers from disk");
    println!("  :mode exact|binary        Select exact or rounded arithmetic");
    println!("  :precision BITS           Binary precision (2..4096)");
    println!("  :round nearest|zero|up|down|away  Binary rounding mode");
    println!("  :display auto|scientific  Numeric display style");
    println!("  :settings                 Show numeric settings");""")

    edit("README.md", """## Roadmap

See `TODO.md`.""", """## Exact advanced mathematics and numeric controls

Exact operations: `pow` (integer exponent), `sqrt` (exact square root),
`root` (integer degree), `fact`, `gcd` and `lcm`. Irrational roots are
explicitly rejected rather than silently approximated. Exponent, factorial,
and root-degree limits protect interactive responsiveness.

The calculator starts in exact mode. To intentionally perform finite-precision
binary arithmetic, use `:mode binary`, `:precision 53` and
`:round nearest|zero|up|down|away`. This mode rounds the operands and result
of the four basic arithmetic operators to a dyadic rational with the specified
significant-bit precision. It has an unbounded exponent range and is **not**
IEEE 754 floating point (no infinities, NaNs or signed zeros). `approx` rounds
only the top value even in exact mode. Settings are session-local and are not
part of undo/redo history. `:display scientific` selects lossless
scientific notation for terminating decimal values; repeating values retain
exact fractional output. `:settings` shows the active controls.

Named registers persist only on explicit `:save` and `:load`. Default path:
Windows `%APPDATA%/rpning/registers.txt`; Linux `$XDG_DATA_HOME/rpning/`
`registers.txt` or `~/.local/share/rpning/registers.txt`. Loading is undoable.
A corrupted register file is rejected without altering memory.

Backend selection and benchmark methodology: `docs/M2-BENCHMARK.md`.

## Roadmap

See `TODO.md`.""")

    # TODO: preserve task names literally, check only completed/grounded goals.
    for old in [
        "- [ ] Benchmark Malachite against Rug",
        "- [ ] Finalize arithmetic backend selection",
        "- [ ] Benchmark integer, rational and real arithmetic",
        "- [ ] Add integer division and remainder",
    ]:
        edit("TODO.md", old, old.replace("[ ]", "[x]"))

    # All edits validated; only now write modified files.
    for path, (content, newline) in pending.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content.replace("\n", newline).encode("utf-8"))
        print(f"written {path.relative_to(ROOT)}")
    print("All exact anchors matched. Run cargo fmt/check/test/Clippy, then cargo build --bin rpn before the oracle.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError) as error:
        sys.exit(f"No changes written: {error}")
