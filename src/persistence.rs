//! Bounded UTF-8 state files with unique temporary files and atomic replacement.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub fn read_text(path: &Path) -> io::Result<String> {
    let mut text = String::new();
    File::open(path)?
        .take(MAX_TEXT_BYTES as u64 + 1)
        .read_to_string(&mut text)?;
    if text.len() > MAX_TEXT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "text file exceeds 1 MiB",
        ));
    }
    Ok(text)
}

pub fn write_text(path: &Path, text: &str) -> io::Result<()> {
    if text.len() > MAX_TEXT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "text file exceeds 1 MiB",
        ));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    for _ in 0..16 {
        let temporary = path.with_extension(format!(
            "{}.{}.tmp",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let result = (|| {
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        return result;
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "cannot allocate temporary state file",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_is_supported_and_failed_write_preserves_original() {
        let path = std::env::temp_dir().join(format!("rpning-atomic-{}.txt", std::process::id()));
        write_text(&path, "first").unwrap();
        write_text(&path, "second").unwrap();
        assert_eq!(read_text(&path).unwrap(), "second");
        assert!(write_text(&path, &"x".repeat(MAX_TEXT_BYTES + 1)).is_err());
        assert_eq!(read_text(&path).unwrap(), "second");
        fs::remove_file(path).unwrap();
    }
}
