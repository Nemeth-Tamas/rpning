//! Layer selection follows emitted event identities, never the NumLock flag.

use crate::config::Config;
use crate::keypad::{Key, Keypad, Outcome};

pub struct Keymap {
    pub config: Config,
    layer: String,
}

impl Keymap {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            layer: "base".to_owned(),
        }
    }
    pub fn layer(&self) -> &str {
        &self.layer
    }
    pub fn handle(
        &mut self,
        pad: &mut Keypad,
        key: &str,
        navigation: bool,
        fallback: Option<Key>,
    ) -> Result<Outcome, String> {
        if fallback == Some(Key::Quit) {
            return Ok(Outcome::Quit);
        }
        if key == "layer" {
            self.layer = if self.layer == "base" { "alt" } else { "base" }.to_owned();
            return Ok(Outcome::Continue);
        }
        let layer = if navigation {
            "nav"
        } else {
            self.layer.as_str()
        };
        let expression = self.config.binding(layer, key).map(str::to_owned);
        match expression.as_deref() {
            Some(action) => self.action(pad, action),
            None => {
                if let Some(key) = fallback {
                    pad.handle(key)
                } else {
                    Ok(Outcome::Continue)
                }
            }
        }
    }
    fn action(&mut self, pad: &mut Keypad, expression: &str) -> Result<Outcome, String> {
        if let Some(layer) = expression.strip_prefix("@layer:") {
            self.layer = layer.to_owned();
            return Ok(Outcome::Continue);
        }
        if let Some(name) = expression.strip_prefix("@shortcut:") {
            let source = self
                .config
                .shortcuts
                .get(name)
                .ok_or_else(|| format!("unknown shortcut '{name}'"))?;
            pad.execute_expression(source)?;
            return Ok(Outcome::Continue);
        }
        let key = match expression {
            "@enter" => Some(Key::Enter),
            "@undo" => Some(Key::Undo),
            "@redo" => Some(Key::Redo),
            "@sign" => Some(Key::ToggleSign),
            "@cancel" => Some(Key::Cancel),
            "@clear" => Some(Key::Clear),
            "@swap" => Some(Key::Swap),
            "@backspace" => Some(Key::Backspace),
            "@quit" => Some(Key::Quit),
            _ => None,
        };
        if let Some(key) = key {
            pad.handle(key)
        } else {
            pad.execute_expression(expression)?;
            Ok(Outcome::Continue)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_and_explicit_layers_execute_macros_atomically() {
        let mut calc = crate::Calculator::new();
        calc.evaluate(": gross 1.27 * ; 100").unwrap();
        let mut pad = Keypad::from_calculator(calc);
        let mut config = Config::default();
        config.bind("nav", "home", "gross").unwrap();
        config.shortcut("bad", "0 /").unwrap();
        config.bind("alt", "9", "@shortcut:bad").unwrap();
        let mut map = Keymap::new(config);
        map.handle(&mut pad, "home", true, None).unwrap();
        assert_eq!(pad.calculator().top().unwrap().to_string(), "127");
        map.handle(&mut pad, "layer", false, None).unwrap();
        assert_eq!(map.layer(), "alt");
        assert!(
            map.handle(&mut pad, "9", false, Some(Key::Digit(9)))
                .is_err()
        );
        assert_eq!(pad.calculator().top().unwrap().to_string(), "127");
        map.handle(&mut pad, "1", false, Some(Key::Digit(1)))
            .unwrap();
        assert_eq!(pad.calculator().top().unwrap().to_string(), "100");
        assert_eq!(
            map.handle(&mut pad, "escape", false, Some(Key::Quit))
                .unwrap(),
            Outcome::Quit
        );
    }
}
