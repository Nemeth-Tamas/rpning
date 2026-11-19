//! Versioned application settings, command shortcuts and explicit keypad layers.

use crate::precision::{NumericSettings, Rounding};
use crate::{Calculator, display::DisplayMode, language, persistence, storage};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub numeric: NumericSettings,
    pub display: DisplayMode,
    pub shortcuts: BTreeMap<String, String>,
    pub bindings: BTreeMap<(String, String), String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Self {
            numeric: NumericSettings::default(),
            display: DisplayMode::Auto,
            shortcuts: BTreeMap::new(),
            bindings: BTreeMap::new(),
        };
        for (layer, key, action) in [
            ("nav", "home", "dup"),
            ("nav", "up", "swap"),
            ("nav", "pageup", "drop"),
            ("nav", "left", "neg"),
            ("nav", "right", "sqrt"),
            ("nav", "end", "@undo"),
            ("nav", "down", "@redo"),
            ("nav", "pagedown", "clear"),
            ("nav", "insert", "depth"),
            ("nav", "delete", "@cancel"),
            ("alt", "7", "dup *"),
            ("alt", "8", "sqrt"),
            ("alt", "9", "1 swap /"),
            ("alt", "4", "swap"),
            ("alt", "5", "dup"),
            ("alt", "6", "drop"),
            ("alt", "1", "@undo"),
            ("alt", "2", "@redo"),
            ("alt", "3", "clear"),
            ("alt", "0", "@sign"),
            ("alt", "decimal", "pi"),
        ] {
            config
                .bindings
                .insert((layer.to_owned(), key.to_owned()), action.to_owned());
        }
        config
    }
}

pub fn state_path(name: &str) -> PathBuf {
    storage::default_register_path().with_file_name(name)
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let path = state_path("settings.txt");
        if !path.exists() {
            return Ok(Self::default());
        }
        Self::decode(&persistence::read_text(&path).map_err(|error| error.to_string())?)
    }
    pub fn save(&self) -> Result<(), String> {
        persistence::write_text(&state_path("settings.txt"), &self.encode())
            .map_err(|error| error.to_string())
    }
    pub fn apply(&self, calc: &mut Calculator) -> Result<(), String> {
        calc.set_precision_bits(self.numeric.bits)
            .map_err(|error| error.to_string())?;
        calc.set_binary_mode(self.numeric.binary_mode);
        calc.set_rounding(self.numeric.rounding);
        calc.set_scientific_display(self.display == DisplayMode::Scientific);
        Ok(())
    }
    pub fn capture(&mut self, calc: &Calculator) {
        self.numeric = calc.settings();
    }
    pub fn encode(&self) -> String {
        let mut text = format!(
            "RPNING-CONFIG-1\nmode\t{}\nprecision\t{}\nround\t{}\ndisplay\t{}\n",
            if self.numeric.binary_mode {
                "binary"
            } else {
                "exact"
            },
            self.numeric.bits,
            self.numeric.rounding.name(),
            self.display.name()
        );
        for (name, expression) in &self.shortcuts {
            text.push_str(&format!("shortcut\t{name}\t{expression}\n"));
        }
        for ((layer, key), expression) in &self.bindings {
            text.push_str(&format!("bind\t{layer}\t{key}\t{expression}\n"));
        }
        text
    }
    pub fn decode(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        if lines.next() != Some("RPNING-CONFIG-1") {
            return Err("unsupported settings file version".to_owned());
        }
        let mut config = Self::default();
        config.bindings.clear();
        let mut seen = std::collections::BTreeSet::new();
        for line in lines {
            let words: Vec<_> = line.split('\t').collect();
            match words.as_slice() {
                ["mode", mode @ ("exact" | "binary")] => {
                    if !seen.insert("mode") {
                        return Err("duplicate setting".to_owned());
                    }
                    config.numeric.binary_mode = *mode == "binary";
                }
                ["precision", bits] => {
                    if !seen.insert("precision") {
                        return Err("duplicate setting".to_owned());
                    }
                    config.numeric.bits = bits.parse().map_err(|_| "invalid bit count")?;
                    if !(2..=4096).contains(&config.numeric.bits) {
                        return Err("precision must be 2..4096".to_owned());
                    }
                }
                ["round", mode] => {
                    if !seen.insert("round") {
                        return Err("duplicate setting".to_owned());
                    }
                    config.numeric.rounding =
                        Rounding::parse(mode).ok_or("invalid rounding mode")?;
                }
                ["display", mode] => {
                    if !seen.insert("display") {
                        return Err("duplicate setting".to_owned());
                    }
                    config.display = DisplayMode::parse(mode).ok_or("invalid display mode")?;
                }
                ["shortcut", name, expression] => {
                    config.shortcut(name, expression)?;
                }
                ["bind", layer, key, expression] => {
                    config.bind(layer, key, expression)?;
                }
                _ => return Err("malformed settings record".to_owned()),
            }
        }
        if seen.len() != 4 {
            return Err("missing required setting".to_owned());
        }
        config.numeric.scientific_display = config.display == DisplayMode::Scientific;
        Ok(config)
    }
    pub fn shortcut(&mut self, name: &str, expression: &str) -> Result<(), String> {
        if !language::valid_name(name) {
            return Err("invalid shortcut name".to_owned());
        }
        validate_expression(expression)?;
        self.shortcuts
            .insert(name.to_owned(), expression.to_owned());
        Ok(())
    }
    pub fn bind(&mut self, layer: &str, key: &str, expression: &str) -> Result<(), String> {
        if !language::valid_name(layer)
            || key.is_empty()
            || key.len() > 32
            || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err("invalid keypad layer or key".to_owned());
        }
        if let Some(action) = expression.strip_prefix('@') {
            if !matches!(
                action,
                "enter"
                    | "undo"
                    | "redo"
                    | "sign"
                    | "cancel"
                    | "clear"
                    | "swap"
                    | "backspace"
                    | "quit"
            ) && !action
                .strip_prefix("layer:")
                .is_some_and(language::valid_name)
                && !action
                    .strip_prefix("shortcut:")
                    .is_some_and(language::valid_name)
            {
                return Err("invalid keypad action".to_owned());
            }
        } else {
            validate_expression(expression)?;
        }
        self.bindings
            .insert((layer.to_owned(), key.to_owned()), expression.to_owned());
        Ok(())
    }
    pub fn binding(&self, layer: &str, key: &str) -> Option<&str> {
        self.bindings
            .get(&(layer.to_owned(), key.to_owned()))
            .map(String::as_str)
    }
}

fn validate_expression(expression: &str) -> Result<(), String> {
    if expression.is_empty() || expression.len() > 4096 || expression.contains(['\n', '\r', '\t']) {
        return Err("shortcut must be one nonempty line, at most 4096 bytes".to_owned());
    }
    let program = language::compile(expression)?;
    if program
        .0
        .iter()
        .any(|instruction| matches!(instruction, language::Instruction::Define(_, _)))
    {
        return Err("shortcut expressions cannot define macros".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_roundtrip_and_validation_preserve_layers_shortcuts_and_precision() {
        let mut config = Config::default();
        config.numeric.bits = 256;
        config.display = DisplayMode::Fixed(8);
        config.shortcut("vat", "1.27 *").unwrap();
        config.bind("nav", "home", "@shortcut:vat").unwrap();
        assert_eq!(Config::decode(&config.encode()).unwrap(), config);
        assert!(Config::decode("RPNING-CONFIG-1\nprecision\t9999\n").is_err());
        assert!(config.bind("nav", "home", "@bogus").is_err());
        assert!(config.shortcut("bad", ": definition 1 ;").is_err());
    }
}
