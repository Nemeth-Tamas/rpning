//! CLI commands for macro inspection and explicit import/export.

use std::path::{Path, PathBuf};

use crate::{Calculator, persistence, storage};

pub fn macro_path() -> PathBuf {
    storage::default_register_path().with_file_name("macros.rpn")
}

pub fn load(calc: &mut Calculator, path: &Path) -> Result<(), String> {
    let source = persistence::read_text(path).map_err(|error| error.to_string())?;
    calc.import_macros(&source)
        .map_err(|error| error.to_string())
}

pub fn save(calc: &Calculator, path: &Path) -> Result<(), String> {
    persistence::write_text(path, &calc.export_macros()).map_err(|error| error.to_string())
}

pub fn is_command(text: &str) -> bool {
    matches!(
        text.split_whitespace().next(),
        Some(
            ":macros"
                | ":see"
                | ":trace"
                | ":alias"
                | ":import"
                | ":export"
                | ":macros-save"
                | ":macros-load"
        )
    )
}

pub fn command(calc: &mut Calculator, text: &str) -> Result<Vec<String>, String> {
    let (command, rest) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
    let rest = rest.trim();
    match command {
        ":macros" if rest.is_empty() => {
            if calc.macros().is_empty() {
                return Ok(vec!["(no macros)".to_owned()]);
            }
            Ok(calc
                .macros()
                .iter()
                .map(|(name, definition)| format!("{name} - {}", definition.description))
                .collect())
        }
        ":see" => {
            let definition = calc
                .macros()
                .get(rest)
                .ok_or("unknown macro; use :macros")?;
            let mut lines = vec![definition.source.clone()];
            lines.extend(
                definition
                    .program
                    .0
                    .iter()
                    .enumerate()
                    .map(|(index, instruction)| format!("{index}: {instruction:?}")),
            );
            Ok(lines)
        }
        ":trace" if !rest.is_empty() => calc
            .evaluate_traced(rest)
            .map_err(|error| error.to_string()),
        ":alias" => {
            let words: Vec<_> = rest.split_whitespace().collect();
            if words.len() != 2
                || !crate::language::valid_name(words[0])
                || words[1].contains([';', '"', '#'])
            {
                return Err("usage: :alias NAME TARGET_WORD".to_owned());
            }
            calc.evaluate(&format!(": {} {} ;", words[0], words[1]))
                .map_err(|error| error.to_string())?;
            Ok(vec![format!("alias: {} -> {}", words[0], words[1])])
        }
        ":import" | ":export" if !rest.is_empty() => {
            let path = Path::new(
                rest.strip_prefix('"')
                    .and_then(|value| value.strip_suffix('"'))
                    .unwrap_or(rest),
            );
            if command == ":import" {
                load(calc, path)?;
            } else {
                save(calc, path)?;
            }
            Ok(vec![format!(
                "{} macros: {}",
                if command == ":import" {
                    "imported"
                } else {
                    "exported"
                },
                path.display()
            )])
        }
        ":macros-save" | ":macros-load" if rest.is_empty() => {
            let path = macro_path();
            if command == ":macros-save" {
                save(calc, &path)?;
            } else {
                load(calc, &path)?;
            }
            Ok(vec![format!(
                "{} macros: {}",
                if command == ":macros-save" {
                    "saved"
                } else {
                    "loaded"
                },
                path.display()
            )])
        }
        _ => Err("invalid macro command; see :help".to_owned()),
    }
}
