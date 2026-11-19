//! Opt-in, focused Windows console input. No global hooks or device capture.

use std::io;

#[cfg(windows)]
pub fn read_line(
    prompt: &str,
    history: &[String],
    completions: &[String],
) -> io::Result<Option<String>> {
    windows::read_line(prompt, history, completions)
}

#[cfg(not(windows))]
pub fn read_line(
    prompt: &str,
    _history: &[String],
    _completions: &[String],
) -> io::Result<Option<String>> {
    use std::io::Write;
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        Ok(None)
    } else {
        Ok(Some(line.trim_end_matches(['\r', '\n']).to_owned()))
    }
}

#[cfg(not(windows))]
pub fn run(_diagnostic: bool) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "keypad console input currently requires Windows; ordinary REPL and --eval remain available",
    ))
}

#[cfg(windows)]
pub fn run(diagnostic: bool) -> io::Result<()> {
    windows::run(diagnostic)
}

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;
    use std::io::{self, IsTerminal, Write};

    use rpning::keypad::{Key, Keypad, Outcome, character_key};
    use rpning::{Calculator, config::Config, keymap::Keymap, programming};

    const KEY_EVENT: u16 = 0x0001;
    const ALT: u32 = 0x0003;
    const CTRL: u32 = 0x000c;
    const NUMLOCK_ON: u32 = 0x0020;
    const ENHANCED_KEY: u32 = 0x0100;
    const ENABLE_EXTENDED_FLAGS: u32 = 0x0080;
    // Process Ctrl+C ourselves, and disable line/echo, mouse/window, Quick Edit,
    // and VT input translation. Preserve all other flags and restore on exit.
    const DISABLED_INPUT_MODES: u32 = 0x0001 | 0x0002 | 0x0004 | 0x0008 | 0x0010 | 0x0040 | 0x0200;

    type Handle = *mut c_void;

    // Windows KEY_EVENT_RECORD: BOOL, three WORDs, WCHAR/CHAR union, DWORD.
    // Every bit pattern is valid; repr(C) supplies the documented ABI layout.
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct KeyEvent {
        down: i32,
        repeats: u16,
        virtual_key: u16,
        scan_code: u16,
        character: u16,
        controls: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    union Event {
        key: KeyEvent,
        // Largest INPUT_RECORD union member (MOUSE_EVENT_RECORD) is 16 bytes,
        // aligned to DWORD. Non-key records are ignored without interpretation.
        raw: [u32; 4],
    }

    #[repr(C)]
    struct InputRecord {
        event_type: u16,
        event: Event,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(which: u32) -> Handle;
        fn GetConsoleMode(handle: Handle, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: Handle, mode: u32) -> i32;
        fn ReadConsoleInputW(
            handle: Handle,
            records: *mut InputRecord,
            length: u32,
            count: *mut u32,
        ) -> i32;
    }

    struct ConsoleInput {
        handle: Handle,
        original_mode: u32,
        restored: bool,
    }

    impl ConsoleInput {
        fn open() -> io::Result<Self> {
            if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "keypad mode requires an interactive Windows console on stdin and stdout; run directly in PowerShell or Windows Terminal",
                ));
            }
            // SAFETY: STD_INPUT_HANDLE is -10 as DWORD. The returned handle is
            // borrowed from the process and is validated by GetConsoleMode.
            let handle = unsafe { GetStdHandle((-10i32) as u32) };
            let mut original_mode = 0;
            // SAFETY: original_mode is writable and the handle is only borrowed.
            if unsafe { GetConsoleMode(handle, &mut original_mode) } == 0 {
                return Err(io::Error::last_os_error());
            }
            let mode = (original_mode | ENABLE_EXTENDED_FLAGS) & !DISABLED_INPUT_MODES;
            // SAFETY: handle was validated above; no output modes are modified.
            if unsafe { SetConsoleMode(handle, mode) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self {
                handle,
                original_mode,
                restored: false,
            })
        }

        fn read_key(&self) -> io::Result<KeyEvent> {
            loop {
                let mut record = InputRecord {
                    event_type: 0,
                    event: Event { raw: [0; 4] },
                };
                let mut count = 0;
                // SAFETY: record has INPUT_RECORD layout, space for one event,
                // and all fields remain valid for every record kind.
                if unsafe { ReadConsoleInputW(self.handle, &mut record, 1, &mut count) } == 0 {
                    return Err(io::Error::last_os_error());
                }
                if count == 1 && record.event_type == KEY_EVENT {
                    // SAFETY: the discriminant confirms KEY_EVENT_RECORD.
                    return Ok(unsafe { record.event.key });
                }
            }
        }

        fn restore(&mut self) -> io::Result<()> {
            if !self.restored {
                // SAFETY: the borrowed console handle remains open for this session.
                if unsafe { SetConsoleMode(self.handle, self.original_mode) } == 0 {
                    return Err(io::Error::last_os_error());
                }
                self.restored = true;
            }
            Ok(())
        }
    }

    impl Drop for ConsoleInput {
        fn drop(&mut self) {
            // Covers input/output errors and unwinding as well as normal exits.
            let _ = self.restore();
        }
    }

    fn mapped_key(event: KeyEvent) -> Option<Key> {
        if event.down == 0 {
            return None;
        }
        if event.controls & CTRL != 0 && event.controls & ALT == 0 {
            return match event.virtual_key {
                0x43 => Some(Key::Quit), // Ctrl+C
                0x5a => Some(Key::Undo), // Ctrl+Z
                0x59 => Some(Key::Redo), // Ctrl+Y
                _ => None,
            };
        }
        if event.controls & (ALT | CTRL) != 0 {
            return None;
        }
        match event.virtual_key {
            0x1b => Some(Key::Quit),
            0x0d => Some(Key::Enter),
            0x08 => Some(Key::Backspace),
            0x09 => Some(Key::Swap),
            // The tested standalone numpad also emits enhanced Delete in
            // navigation mode. This cancels only the pending entry.
            0x2e if event.controls & ENHANCED_KEY != 0 => Some(Key::Cancel),
            _ => char::from_u32(u32::from(event.character)).and_then(character_key),
        }
    }

    fn diagnostic_line(event: KeyEvent) -> String {
        let character = char::from_u32(u32::from(event.character));
        format!(
            "{} repeat={} vk=0x{:04X} scan=0x{:04X} unicode=U+{:04X} char={:?} flags=0x{:04X} numlock={} enhanced={} mapped={:?}",
            if event.down != 0 { "down" } else { "up  " },
            event.repeats,
            event.virtual_key,
            event.scan_code,
            event.character,
            character,
            event.controls,
            event.controls & NUMLOCK_ON != 0,
            event.controls & ENHANCED_KEY != 0,
            mapped_key(event),
        )
    }

    fn key_identity(event: KeyEvent) -> Option<(String, bool)> {
        if event.down == 0 || event.controls & (CTRL | ALT) != 0 {
            return None;
        }
        let navigation = match event.virtual_key {
            0x24 => Some("home"),
            0x26 => Some("up"),
            0x21 => Some("pageup"),
            0x25 => Some("left"),
            0x27 => Some("right"),
            0x23 => Some("end"),
            0x28 => Some("down"),
            0x22 => Some("pagedown"),
            0x2d => Some("insert"),
            0x2e => Some("delete"),
            _ => None,
        };
        if let Some(key) = navigation {
            return Some((key.to_owned(), true));
        }
        let special = match event.virtual_key {
            0x0d => Some("enter"),
            0x08 => Some("backspace"),
            0x09 => Some("tab"),
            0x71 => Some("layer"),
            _ => None,
        };
        if let Some(key) = special {
            return Some((key.to_owned(), false));
        }
        let character = char::from_u32(u32::from(event.character))?;
        let key = match character {
            '.' | ',' => "decimal".to_owned(),
            '+' => "add".to_owned(),
            '-' => "subtract".to_owned(),
            '*' => "multiply".to_owned(),
            '/' => "divide".to_owned(),
            c if c.is_ascii_alphanumeric() => c.to_ascii_lowercase().to_string(),
            _ => return None,
        };
        Some((key, false))
    }

    fn clipped(text: &str) -> String {
        const LIMIT: usize = 56;
        if text.chars().count() > LIMIT {
            format!("{}...", text.chars().take(LIMIT).collect::<String>())
        } else {
            text.to_owned()
        }
    }

    fn show_state(out: &mut impl Write, pad: &Keypad, map: &Keymap) -> io::Result<()> {
        let calc = pad.calculator();
        let top = calc
            .top()
            .map(|value| map.config.display.format(value, calc.settings().rounding))
            .unwrap_or_else(|| "(empty)".to_owned());
        writeln!(
            out,
            "depth={}  X={}  entry={}  layer={}",
            calc.depth(),
            clipped(&top),
            clipped(pad.pending()),
            map.layer()
        )?;
        out.flush()
    }

    pub fn run(diagnostic: bool) -> io::Result<()> {
        let mut input = ConsoleInput::open()?;
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let result = session(&input, &mut out, diagnostic);
        let restored = input.restore();
        result.and(restored)
    }

    fn session(input: &ConsoleInput, out: &mut impl Write, diagnostic: bool) -> io::Result<()> {
        if diagnostic {
            writeln!(
                out,
                "rpning keypad diagnostic: focused console input only. Esc or Ctrl+C exits."
            )?;
            writeln!(
                out,
                "Press each key with NumLock ON, then OFF; include decimal, Enter, operators and Backspace."
            )?;
            writeln!(
                out,
                "Events describe the console host's translation; no physical-device identity is available."
            )?;
        } else {
            writeln!(
                out,
                "rpning keypad: digits, . or ,; + - * / execute immediately."
            )?;
            writeln!(
                out,
                "Enter commits entry, or duplicates X when entry is empty. Backspace edits entry."
            )?;
            writeln!(
                out,
                "S=sign, C=clear stack, Tab=swap, Delete=cancel entry, Ctrl+Z/Y=undo/redo, Esc=exit."
            )?;
            writeln!(
                out,
                "Use NumLock ON. Unmapped keys do nothing; long displayed values are clipped, stored values stay exact."
            )?;
        }
        out.flush()?;
        let config = if diagnostic {
            Config::default()
        } else {
            Config::load().map_err(io::Error::other)?
        };
        let mut calc = Calculator::new();
        config.apply(&mut calc).map_err(io::Error::other)?;
        if !diagnostic && programming::macro_path().exists() {
            programming::load(&mut calc, &programming::macro_path()).map_err(io::Error::other)?;
        }
        calc.clear_history();
        let mut pad = Keypad::from_calculator(calc);
        let mut map = Keymap::new(config);
        if !diagnostic {
            writeln!(
                out,
                "Navigation keys use the nav layer; F2 toggles base/alt. Use :bindings in the REPL to inspect mappings."
            )?;
            show_state(out, &pad, &map)?;
        }
        loop {
            let event = input.read_key()?;
            if diagnostic {
                writeln!(out, "{}", diagnostic_line(event))?;
                out.flush()?;
                if mapped_key(event) == Some(Key::Quit) {
                    break;
                }
            } else if event.down != 0 {
                let fallback = mapped_key(event);
                let identity = key_identity(event);
                if fallback.is_none() && identity.is_none() {
                    continue;
                }
                let (key, navigation) = identity.unwrap_or_else(|| (String::new(), false));
                for _ in 0..event.repeats {
                    match map.handle(&mut pad, &key, navigation, fallback) {
                        Ok(Outcome::Quit) => return Ok(()),
                        Ok(Outcome::Continue) => {}
                        Err(error) => {
                            writeln!(out, "error: {error}")?;
                            break;
                        }
                    }
                }
                show_state(out, &pad, &map)?;
            }
        }
        Ok(())
    }

    pub fn read_line(
        prompt: &str,
        history: &[String],
        completions: &[String],
    ) -> io::Result<Option<String>> {
        let mut input = ConsoleInput::open()?;
        let stdout = io::stdout();
        let mut out = stdout.lock();
        let result = edit_line(&input, &mut out, prompt, history, completions);
        let restored = input.restore();
        match (result, restored) {
            (Ok(line), Ok(())) => Ok(line),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    fn redraw(
        out: &mut impl Write,
        prompt: &str,
        buffer: &[char],
        cursor: usize,
    ) -> io::Result<()> {
        let start = cursor.saturating_sub(55);
        let end = (start + 60).min(buffer.len());
        let visible: String = buffer[start..end].iter().collect();
        let before: String = buffer[start..cursor].iter().collect();
        write!(out, "\r{prompt}{:64}\r{prompt}{before}", visible)?;
        out.flush()
    }

    fn edit_line(
        input: &ConsoleInput,
        out: &mut impl Write,
        prompt: &str,
        history: &[String],
        completions: &[String],
    ) -> io::Result<Option<String>> {
        let mut buffer = Vec::new();
        let mut cursor = 0;
        let mut history_index = history.len();
        let mut draft = String::new();
        let mut search: Option<(String, usize)> = None;
        let mut high_surrogate = None;
        redraw(out, prompt, &buffer, cursor)?;
        loop {
            let event = input.read_key()?;
            if event.down == 0 {
                continue;
            }
            let control = event.controls & CTRL != 0 && event.controls & ALT == 0;
            if control {
                match event.virtual_key {
                    0x43 => {
                        writeln!(out, "^C")?;
                        return Ok(Some(String::new()));
                    }
                    0x44 if buffer.is_empty() => {
                        writeln!(out)?;
                        return Ok(None);
                    }
                    0x55 => {
                        buffer.clear();
                        cursor = 0;
                        search = None;
                    }
                    0x52 => {
                        let (needle, before) =
                            search.get_or_insert_with(|| (buffer.iter().collect(), history.len()));
                        if let Some(index) = (0..*before)
                            .rev()
                            .find(|index| history[*index].contains(needle.as_str()))
                        {
                            buffer = history[index].chars().collect();
                            cursor = buffer.len();
                            *before = index;
                        }
                    }
                    _ => {}
                }
            } else {
                for _ in 0..event.repeats {
                    match event.virtual_key {
                        0x0d => {
                            redraw(out, prompt, &buffer, buffer.len())?;
                            writeln!(out)?;
                            return Ok(Some(buffer.iter().collect()));
                        }
                        0x1b => {
                            buffer.clear();
                            cursor = 0;
                            search = None;
                        }
                        0x08 if cursor > 0 => {
                            cursor -= 1;
                            buffer.remove(cursor);
                            search = None;
                        }
                        0x2e if cursor < buffer.len() => {
                            buffer.remove(cursor);
                            search = None;
                        }
                        0x25 => cursor = cursor.saturating_sub(1),
                        0x27 => cursor = (cursor + 1).min(buffer.len()),
                        0x24 => cursor = 0,
                        0x23 => cursor = buffer.len(),
                        0x26 => {
                            if history_index == history.len() {
                                draft = buffer.iter().collect();
                            }
                            history_index = history_index.saturating_sub(1);
                            if let Some(line) = history.get(history_index) {
                                buffer = line.chars().collect();
                                cursor = buffer.len();
                            }
                        }
                        0x28 => {
                            history_index = (history_index + 1).min(history.len());
                            let line = history.get(history_index).unwrap_or(&draft);
                            buffer = line.chars().collect();
                            cursor = buffer.len();
                        }
                        0x09 => {
                            let start = buffer[..cursor]
                                .iter()
                                .rposition(|c| c.is_whitespace())
                                .map_or(0, |index| index + 1);
                            let prefix: String = buffer[start..cursor].iter().collect();
                            let matches: Vec<_> = completions
                                .iter()
                                .filter(|word| word.starts_with(&prefix))
                                .collect();
                            if matches.len() == 1 {
                                let replacement: Vec<_> = matches[0].chars().collect();
                                cursor = start + replacement.len();
                                buffer.splice(start..(start + prefix.chars().count()), replacement);
                            } else if !matches.is_empty() {
                                writeln!(
                                    out,
                                    "\n{}",
                                    matches
                                        .iter()
                                        .take(30)
                                        .map(|word| word.as_str())
                                        .collect::<Vec<_>>()
                                        .join("  ")
                                )?;
                            }
                        }
                        _ => {
                            let unit = event.character;
                            if (0xd800..=0xdbff).contains(&unit) {
                                high_surrogate = Some(unit);
                                continue;
                            }
                            let character = if let Some(high) = high_surrogate.take() {
                                if (0xdc00..=0xdfff).contains(&unit) {
                                    char::from_u32(
                                        0x10000
                                            + ((u32::from(high) - 0xd800) << 10)
                                            + u32::from(unit)
                                            - 0xdc00,
                                    )
                                } else {
                                    char::from_u32(u32::from(unit))
                                }
                            } else {
                                char::from_u32(u32::from(unit))
                            };
                            if let Some(character) = character.filter(|c| !c.is_control())
                                && buffer.len() < 4096
                            {
                                buffer.insert(cursor, character);
                                cursor += 1;
                                search = None;
                            }
                        }
                    }
                }
            }
            redraw(out, prompt, &buffer, cursor)?;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn event(virtual_key: u16, character: char, controls: u32) -> KeyEvent {
            KeyEvent {
                down: 1,
                repeats: 1,
                virtual_key,
                character: character as u16,
                controls,
                ..KeyEvent::default()
            }
        }

        #[test]
        fn bindings_match_windows_console_abi() {
            assert_eq!(std::mem::size_of::<KeyEvent>(), 16);
            assert_eq!(std::mem::offset_of!(KeyEvent, controls), 12);
            assert_eq!(std::mem::size_of::<InputRecord>(), 20);
            assert_eq!(std::mem::offset_of!(InputRecord, event), 4);
            assert_eq!(std::mem::align_of::<InputRecord>(), 4);
        }

        #[test]
        fn console_translation_respects_characters_modifiers_and_release() {
            assert_eq!(mapped_key(event(0x6e, ',', NUMLOCK_ON)), Some(Key::Decimal));
            assert_eq!(
                mapped_key(event(0x61, '1', NUMLOCK_ON)),
                Some(Key::Digit(1))
            );
            assert_eq!(
                mapped_key(event(0x0d, '\r', ENHANCED_KEY)),
                Some(Key::Enter)
            );
            assert_eq!(mapped_key(event(0x23, '\0', 0)), None); // NumLock-off End
            assert_eq!(mapped_key(event(0x2e, '\0', 0)), None); // NumLock-off decimal
            assert_eq!(
                mapped_key(event(0x2e, '\0', ENHANCED_KEY)),
                Some(Key::Cancel)
            );
            assert_eq!(mapped_key(event(0x43, '\u{3}', CTRL)), Some(Key::Quit));
            assert_eq!(mapped_key(event(0x5a, '\u{1a}', CTRL)), Some(Key::Undo));
            assert_eq!(mapped_key(event(0x59, '\u{19}', CTRL)), Some(Key::Redo));
            assert_eq!(mapped_key(event(0x61, '1', ALT)), None);
            assert_eq!(mapped_key(event(0x61, '1', ALT | CTRL)), None);
            let mut released = event(0x61, '1', NUMLOCK_ON);
            released.down = 0;
            assert_eq!(mapped_key(released), None);
        }

        #[test]
        fn diagnostics_include_reported_codes_without_guessing_device_identity() {
            let mut key = event(0x6e, ',', NUMLOCK_ON);
            key.scan_code = 0x53;
            key.repeats = 3;
            let text = diagnostic_line(key);
            for expected in [
                "down",
                "repeat=3",
                "vk=0x006E",
                "scan=0x0053",
                "U+002C",
                "numlock=true",
                "Decimal",
            ] {
                assert!(text.contains(expected), "{text}");
            }
        }

        #[test]
        fn observed_numcalc_events_work_without_the_numlock_flag() {
            for (vk, character, flags, expected) in [
                (0x37, '7', 0, Some(Key::Digit(7))),
                (0x30, '0', 0, Some(Key::Digit(0))),
                (0x38, '*', 0x10, Some(Key::Multiply)),
                (0xbb, '+', 0x10, Some(Key::Add)),
                (0xbf, '/', 0, Some(Key::Divide)),
                (0xbd, '-', 0, Some(Key::Subtract)),
                (0xbe, '.', 0, Some(Key::Decimal)),
                (0x24, '\0', 0x100, None),
                (0x2e, '\0', 0x100, Some(Key::Cancel)),
            ] {
                assert_eq!(mapped_key(event(vk, character, flags)), expected);
            }
        }
    }
}
