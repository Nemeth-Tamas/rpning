//! Application commands that do not belong to numeric evaluation.

use crate::display::DisplayMode;
use crate::precision::Rounding;
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
    words.extend(crate::extended::WORDS.split_whitespace().map(str::to_owned));
    words.extend(calc.macros().keys().cloned());
    for (name, _) in calc.registers() {
        words.push(format!("rcl:{name}"));
        words.push(format!("sto:{name}"));
    }
    words.sort();
    words.dedup();
    words
}

pub fn is_numeric_command(line: &str) -> bool {
    matches!(
        line.split_whitespace().next(),
        Some(":settings" | ":mode" | ":precision" | ":round" | ":display")
    )
}

/// Read-only discovery is shared by the TUI, REPL and piped commands.
pub fn setting_help(calc: &Calculator, config: &Config, command: &str) -> Option<Vec<String>> {
    let settings = calc.settings();
    let (current, options) = match command {
        ":mode" => (
            if settings.binary_mode {
                "binary"
            } else {
                "exact"
            }
            .to_owned(),
            "exact | binary (rounded arithmetic; not an integer display base)".to_owned(),
        ),
        ":precision" => (
            format!("{} bits", settings.bits),
            "any integer from 2 through 4096 bits (transcendental functions: at most 512)"
                .to_owned(),
        ),
        ":round" => (
            settings.rounding.name().to_owned(),
            "nearest (ties to even) | zero | up | down | away".to_owned(),
        ),
        ":display" => (
            config.display.name(),
            "auto | scientific | fraction | binary | octal | hex | fixed:N (N=0..128)".to_owned(),
        ),
        _ => return None,
    };
    Some(vec![
        format!("{command}: current = {current}"),
        format!("Allowed: {options}"),
        format!(
            "Usage: {command} VALUE; no argument leaves the setting unchanged. Tab lists choices."
        ),
    ])
}

/// Complete a setting argument, otherwise a calculator word/command.
pub fn completion_candidates(calc: &Calculator, before_cursor: &str) -> Vec<String> {
    let words: Vec<_> = before_cursor.split_whitespace().collect();
    let argument =
        words.len() == 2 || (words.len() == 1 && before_cursor.ends_with(char::is_whitespace));
    let prefix = if before_cursor.ends_with(char::is_whitespace) {
        ""
    } else {
        words.last().copied().unwrap_or("")
    };
    let candidates: Vec<String> = if argument {
        match words.first().copied() {
            Some(":display") => ["auto", "scientific", "fraction", "binary", "octal", "hex"]
                .into_iter()
                .map(str::to_owned)
                .chain((0..=128).map(|n| format!("fixed:{n}")))
                .collect(),
            Some(":mode") => ["exact", "binary"].into_iter().map(str::to_owned).collect(),
            Some(":round") => ["nearest", "zero", "up", "down", "away"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            Some(":precision") => [2, 8, 16, 32, 53, 64, 128, 256, 512, 1024, 2048, 4096]
                .into_iter()
                .map(|n| n.to_string())
                .collect(),
            _ => completions(calc),
        }
    } else {
        completions(calc)
    };
    candidates
        .into_iter()
        .filter(|word| word.starts_with(prefix))
        .collect()
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

pub fn configure_numeric(
    calc: &mut Calculator,
    config: &mut Config,
    command: &str,
) -> Result<String, String> {
    let words: Vec<&str> = command.split_whitespace().collect();
    if let [name] = words.as_slice()
        && let Some(lines) = setting_help(calc, config, name)
    {
        return Ok(lines.join("\n"));
    }
    match words.as_slice() {
        [":settings"] => {
            let settings = calc.settings();
            Ok(format!(
                "mode={} precision={} bits rounding={} display={}",
                if settings.binary_mode {
                    "binary"
                } else {
                    "exact"
                },
                settings.bits,
                settings.rounding.name(),
                config.display.name(),
            ))
        }
        [":mode", mode @ ("exact" | "binary")] => {
            calc.set_binary_mode(*mode == "binary");
            Ok(format!("numeric mode: {mode}"))
        }
        [":precision", bits] => {
            let bits = bits
                .parse::<u64>()
                .map_err(|_| "invalid bit count".to_owned())?;
            calc.set_precision_bits(bits)
                .map_err(|error| error.to_string())?;
            Ok(format!("precision: {bits} bits"))
        }
        [":round", mode] => {
            let rounding = Rounding::parse(mode).ok_or("rounding: nearest|zero|up|down|away")?;
            calc.set_rounding(rounding);
            Ok(format!("rounding: {}", rounding.name()))
        }
        [":display", mode] => {
            config.display = DisplayMode::parse(mode)
                .ok_or("display: auto|scientific|fraction|binary|octal|hex|fixed:N (N=0..128)")?;
            calc.set_scientific_display(config.display == DisplayMode::Scientific);
            Ok(format!("display: {}", config.display.name()))
        }
        _ => Err("invalid settings command; see :help".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_queries_are_read_only_and_completion_covers_every_display_choice() {
        let mut calc = Calculator::new();
        let mut config = Config::default();
        calc.evaluate("1 3 /").unwrap();
        configure_numeric(&mut calc, &mut config, ":display fixed:3").unwrap();
        configure_numeric(&mut calc, &mut config, ":precision 90").unwrap();
        let saved = config.clone();
        let settings = calc.settings();
        for (command, current) in [
            (":display", "fixed:3"),
            (":precision", "90 bits"),
            (":mode", "exact"),
            (":round", "nearest"),
        ] {
            let help = configure_numeric(&mut calc, &mut config, command).unwrap();
            assert!(
                help.contains(current) && help.contains("Allowed:"),
                "{help}"
            );
        }
        assert_eq!(config, saved);
        assert_eq!(calc.settings(), settings);
        assert_eq!(calc.top().unwrap().to_string(), "1/3");
        let options = completion_candidates(&calc, ":display ");
        assert_eq!(options.len(), 135);
        for option in options {
            assert!(DisplayMode::parse(&option).is_some(), "{option}");
        }
        assert_eq!(completion_candidates(&calc, ":display h"), ["hex"]);
        assert_eq!(completion_candidates(&calc, ":mode b"), ["binary"]);
        assert_eq!(completion_candidates(&calc, ":round d"), ["down"]);
        assert!(configure_numeric(&mut calc, &mut config, ":display bogus").is_err());
        assert_eq!(config, saved);
    }
}
