use std::process::Command;

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
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "3/10");
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
