use std::process::{Command, Stdio};

#[test]
fn keypad_modes_reject_piped_input_without_hanging_or_changing_repl() {
    for flag in ["--keypad", "--keypad-diagnose"] {
        let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
            .arg(flag)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Windows"));
        assert!(output.stdout.is_empty());
    }
    let normal = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .args(["--eval", "0.1 0.2 +"])
        .output()
        .unwrap();
    assert!(normal.status.success());
    assert_eq!(String::from_utf8_lossy(&normal.stdout).trim(), "0.3");
}

#[test]
fn help_exposes_keypad_modes_and_extra_arguments_are_rejected() {
    let help = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--keypad-diagnose"));
    let invalid = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .args(["--keypad", "unexpected"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
}
