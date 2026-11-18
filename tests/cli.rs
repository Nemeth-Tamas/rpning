use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn cli_evaluates_expression() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .args(["--eval", "15 7 + 3 *"])
        .output()
        .expect("failed to start rpn binary");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "66");
}

#[test]
fn cli_preserves_exact_precision() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .args(["--eval", "0.1 0.2 +"])
        .output()
        .expect("failed to start rpn binary");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "0.3");
}

#[test]
fn cli_formats_terminating_and_recurring_fractions() {
    let cases = [
        ("10 4 /", "2.5"),
        ("1 8 /", "0.125"),
        ("1 3 /", "1/3"),
        ("1 6 /", "1/6"),
        ("-5 2 /", "-2.5"),
        ("0.1 0.2 +", "0.3"),
    ];

    for (expression, expected) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
            .args(["--eval", expression])
            .output()
            .expect("failed to start rpn binary");

        assert!(output.status.success(), "failed expression: {expression}");

        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            expected,
            "expression: {expression}"
        );
    }
}

#[test]
fn cli_supports_integer_division_and_remainder() {
    let cases = [
        ("17 5 div", "3"),
        ("17 5 mod", "2"),
        ("-17 5 div", "-3"),
        ("-17 5 mod", "-2"),
        ("17 5 /", "3.4"),
    ];

    for (expression, expected) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
            .args(["--eval", expression])
            .output()
            .expect("failed to start rpn binary");

        assert!(output.status.success(), "failed expression: {expression}");

        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            expected,
            "expression: {expression}"
        );
    }
}

#[test]
fn cli_exits_with_error_on_division_by_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .args(["--eval", "10 0 /"])
        .output()
        .expect("failed to start rpn binary");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("division by zero"));
}

#[test]
fn cli_exits_with_error_on_invalid_token() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .args(["--eval", "3 nonsense +"])
        .output()
        .expect("failed to start rpn binary");

    assert!(!output.status.success());
}

#[test]
fn cli_accepts_attached_operator() {
    let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .args(["--eval", "5 3+"])
        .output()
        .expect("failed to start rpn binary");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "8");
}

#[test]
fn stdin_supports_register_deletion_and_clear() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start rpn binary");

    let mut stdin = child.stdin.take().expect("missing child stdin");

    stdin
        .write_all(
            b"10 sto:net\n20 sto:tax\n:registers\ndel:net\n:registers\n:undo\n:registers\nclearmem\n:registers\n:undo\n:registers\n",
        )
        .expect("failed to write commands");

    drop(stdin);

    let output = child.wait_with_output().expect("failed to read output");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [
            "10",
            "20",
            "net = 10",
            "tax = 20",
            "20",
            "tax = 20",
            "20",
            "net = 10",
            "tax = 20",
            "20",
            "(no registers)",
            "20",
            "net = 10",
            "tax = 20",
        ]
    );
}

#[test]
fn stdin_supports_named_registers_and_history() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start rpn binary");

    let mut stdin = child.stdin.take().expect("missing child stdin");

    stdin
        .write_all(b"10 sto:tax\nclear\nrcl:tax\n:registers\n:undo\n:registers\n:redo\n")
        .expect("failed to write commands");

    drop(stdin);

    let output = child.wait_with_output().expect("failed to read output");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [
            "10", "(empty)", "10", "tax = 10", "(empty)", "tax = 10", "10",
        ]
    );
}

#[test]
fn stdin_supports_undo_and_redo() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start rpn binary");

    let mut stdin = child.stdin.take().expect("missing child stdin");

    stdin
        .write_all(b"5\n3+\n:undo\n:undo\n:redo\n:stack\n:redo\n")
        .expect("failed to write commands");

    drop(stdin);

    let output = child.wait_with_output().expect("failed to read output");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        ["5", "8", "5", "(empty)", "5", "[0] 5", "8"]
    );
}

#[test]
fn stdin_supports_enter_separated_calculation() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start rpn binary");

    let mut stdin = child.stdin.take().expect("missing child stdin");

    stdin
        .write_all(b"5\n3+\n2*\n:stack\n")
        .expect("failed to write commands");

    drop(stdin);

    let output = child.wait_with_output().expect("failed to read output");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        ["5", "8", "16", "[0] 16",]
    );
}
