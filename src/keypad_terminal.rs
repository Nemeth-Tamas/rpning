//! Opt-in, focused Windows console input. No global hooks or device capture.

use std::io;

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
    use rpning::number::format_number;

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
            // An enhanced Delete is the navigation key. Do not interpret the
            // keypad's NumLock-off Delete as a destructive editing shortcut.
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

    fn clipped(text: &str) -> String {
        const LIMIT: usize = 56;
        if text.chars().count() > LIMIT {
            format!("{}...", text.chars().take(LIMIT).collect::<String>())
        } else {
            text.to_owned()
        }
    }

    fn show_state(out: &mut impl Write, pad: &Keypad) -> io::Result<()> {
        let calc = pad.calculator();
        let top = calc
            .top()
            .map(format_number)
            .unwrap_or_else(|| "(empty)".to_owned());
        writeln!(
            out,
            "depth={}  X={}  entry={}",
            calc.depth(),
            clipped(&top),
            clipped(pad.pending())
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
        let mut pad = Keypad::new();
        if !diagnostic {
            show_state(out, &pad)?;
        }
        loop {
            let event = input.read_key()?;
            if diagnostic {
                writeln!(out, "{}", diagnostic_line(event))?;
                out.flush()?;
                if mapped_key(event) == Some(Key::Quit) {
                    break;
                }
            } else if let Some(key) = mapped_key(event) {
                for _ in 0..event.repeats {
                    match pad.handle(key) {
                        Ok(Outcome::Quit) => return Ok(()),
                        Ok(Outcome::Continue) => {}
                        Err(error) => {
                            writeln!(out, "error: {error}")?;
                            break;
                        }
                    }
                }
                show_state(out, &pad)?;
            }
        }
        Ok(())
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
    }
}
