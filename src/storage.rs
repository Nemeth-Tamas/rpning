//! Versioned, lossless persistence for exact named registers.
//!
//! Only explicit :save and :load use this module. The format is plain UTF-8:
//! a version header followed by name, tab, canonical Rational value per line.

use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::number::{Number, parse_literal};

const HEADER: &str = "RPNING-REGISTERS-1";
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_REGISTERS: usize = 10_000;

pub fn default_register_path() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(path) = env::var_os("APPDATA") {
            return PathBuf::from(path).join("rpning").join("registers.txt");
        }
    }
    if let Some(path) = env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(path).join("rpning").join("registers.txt");
    }
    if let Some(path) = env::var_os("HOME") {
        return PathBuf::from(path)
            .join(".local")
            .join("share")
            .join("rpning")
            .join("registers.txt");
    }
    PathBuf::from("rpning-registers.txt")
}

pub fn load(path: &Path) -> io::Result<BTreeMap<String, Number>> {
    let size = fs::metadata(path)?.len();
    if size > MAX_FILE_BYTES {
        return Err(invalid("register file exceeds 16 MiB"));
    }
    let contents = fs::read_to_string(path)?;
    let mut lines = contents.lines();
    if lines.next() != Some(HEADER) {
        return Err(invalid("unsupported register file version"));
    }
    let mut registers = BTreeMap::new();
    for line in lines {
        let (name, value_text) = line
            .split_once('\t')
            .ok_or_else(|| invalid("malformed register record"))?;
        if !valid_name(name) {
            return Err(invalid("invalid register name"));
        }
        let value =
            parse_literal(value_text).ok_or_else(|| invalid("invalid rational register value"))?;
        if registers.insert(name.to_owned(), value).is_some() {
            return Err(invalid("duplicate register name"));
        }
        if registers.len() > MAX_REGISTERS {
            return Err(invalid("too many registers"));
        }
    }
    Ok(registers)
}

/// Write a temporary file in the same directory, sync it, then rename it.
/// A failed write leaves the original destination unchanged.
pub fn save<'a>(
    path: &Path,
    registers: impl Iterator<Item = (&'a str, &'a Number)>,
) -> io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let mut data = String::from(HEADER);
    data.push('\n');
    let mut count = 0;
    for (name, value) in registers {
        if !valid_name(name) {
            return Err(invalid("invalid register name"));
        }
        count += 1;
        if count > MAX_REGISTERS {
            return Err(invalid("too many registers"));
        }
        data.push_str(name);
        data.push('\t');
        data.push_str(&value.to_string());
        data.push('\n');
        if data.len() as u64 > MAX_FILE_BYTES {
            return Err(invalid("register file exceeds 16 MiB"));
        }
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let write_result = (|| {
        let mut file = File::create(&temporary)?;
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}

fn valid_name(name: &str) -> bool {
    name.len() <= 64
        && name
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && name
            .bytes()
            .skip(1)
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn invalid(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

#[cfg(test)]
mod tests {
    use super::{load, save};
    use malachite::Rational;
    use std::collections::BTreeMap;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_file() -> std::path::PathBuf {
        let number = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "rpning-register-test-{}-{number}.txt",
            std::process::id()
        ))
    }

    #[test]
    fn exact_values_survive_serialization() {
        let path = unique_file();
        let data = BTreeMap::from([
            (
                "huge".to_owned(),
                "123456789012345678901234567890"
                    .parse::<Rational>()
                    .unwrap(),
            ),
            ("third".to_owned(), "-1/3".parse::<Rational>().unwrap()),
        ]);
        save(&path, data.iter().map(|(k, v)| (k.as_str(), v))).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded, data);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_files_fail_without_partial_success() {
        let path = unique_file();
        fs::write(&path, "RPNING-REGISTERS-1\nvalid\t1/3\nbad.name\t2\n").unwrap();
        assert!(load(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}
