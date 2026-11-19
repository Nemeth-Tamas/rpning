//! Application commands that do not belong to numeric evaluation.

use crate::{Calculator, config::Config, history::History};

pub fn is_command(line: &str) -> bool {
    matches!(
        line.split_whitespace().next(),
        Some(
            ":settings-save"
                | ":settings-load"
                | ":history"
                | ":history-save"
                | ":history-load"
                | ":complete"
                | ":shortcut"
                | ":shortcuts"
                | ":do"
                | ":bind"
                | ":unbind"
                | ":bindings"
        )
    )
}

pub fn completions(calc: &Calculator) -> Vec<String> {
    let mut words: Vec<String> = "+ - * / div mod pow sqrt root fact gcd lcm pi e exp ln log2 log10 sin cos tan rad deg approx dup drop swap over rot nip tuck clear depth pick roll clearmem abs neg not if else then times while do end :stack :registers :save :load :undo :redo :mode :precision :round :display :settings :settings-save :settings-load :history :history-save :history-load :complete :shortcut :shortcuts :do :bind :unbind :bindings :macros :see :trace :alias :import :export :macros-save :macros-load :help :quit".split_whitespace().map(str::to_owned).collect();
    words.extend(calc.macros().keys().cloned());
    for (name, _) in calc.registers() {
        words.push(format!("rcl:{name}"));
        words.push(format!("sto:{name}"));
    }
    words.sort();
    words.dedup();
    words
}

pub fn command(
    calc: &mut Calculator,
    config: &mut Config,
    history: &mut History,
    line: &str,
) -> Result<Vec<String>, String> {
    let (command, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    let rest = rest.trim();
    match command {
        ":settings-save" if rest.is_empty() => {
            config.capture(calc);
            config.save()?;
            Ok(vec!["saved settings".to_owned()])
        }
        ":settings-load" if rest.is_empty() => {
            let loaded = Config::load()?;
            loaded.apply(calc)?;
            *config = loaded;
            Ok(vec!["loaded settings".to_owned()])
        }
        ":history-save" if rest.is_empty() => {
            history.save()?;
            Ok(vec!["saved history".to_owned()])
        }
        ":history-load" if rest.is_empty() => {
            *history = History::load()?;
            Ok(vec!["loaded history".to_owned()])
        }
        ":history" => Ok(history
            .search(rest)
            .map(|(index, text)| format!("{index}: {text}"))
            .collect()),
        ":complete" => Ok(completions(calc)
            .into_iter()
            .filter(|word| word.starts_with(rest))
            .collect()),
        ":shortcuts" if rest.is_empty() => Ok(config
            .shortcuts
            .iter()
            .map(|(name, expression)| format!("{name} = {expression}"))
            .collect()),
        ":shortcut" => {
            let (name, expression) = rest
                .split_once(char::is_whitespace)
                .ok_or("usage: :shortcut NAME EXPRESSION")?;
            config.shortcut(name, expression.trim())?;
            Ok(vec![format!("shortcut: {name}")])
        }
        ":do" => {
            let expression = config.shortcuts.get(rest).ok_or("unknown shortcut")?;
            calc.evaluate(expression)
                .map_err(|error| error.to_string())?;
            Ok(calc
                .top()
                .map(|value| config.display.format(value, calc.settings().rounding))
                .into_iter()
                .collect())
        }
        ":bindings" if rest.is_empty() => Ok(config
            .bindings
            .iter()
            .map(|((layer, key), expression)| format!("{layer}.{key} = {expression}"))
            .collect()),
        ":bind" => {
            let (layer, rest) = rest
                .split_once(char::is_whitespace)
                .ok_or("usage: :bind LAYER KEY ACTION_OR_EXPRESSION")?;
            let (key, expression) = rest
                .trim_start()
                .split_once(char::is_whitespace)
                .ok_or("usage: :bind LAYER KEY ACTION_OR_EXPRESSION")?;
            config.bind(layer, key, expression.trim())?;
            Ok(vec![format!("binding: {layer}.{key}")])
        }
        ":unbind" => {
            let words: Vec<_> = rest.split_whitespace().collect();
            if words.len() != 2 {
                return Err("usage: :unbind LAYER KEY".to_owned());
            }
            if config
                .bindings
                .remove(&(words[0].to_owned(), words[1].to_owned()))
                .is_none()
            {
                return Err("unknown binding".to_owned());
            }
            Ok(vec![format!("unbound: {}.{}", words[0], words[1])])
        }
        _ => Err("invalid workflow command; see :help".to_owned()),
    }
}
