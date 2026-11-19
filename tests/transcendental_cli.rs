use std::io::Write;
use std::process::{Command, Stdio};

fn script(input: &str) -> (bool, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to launch rpn");
    child
        .stdin
        .take()
        .expect("missing stdin")
        .write_all(input.as_bytes())
        .expect("write to child");
    let output = child.wait_with_output().expect("collect output");
    (
        output.status.success(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

#[test]
fn exact_identities_and_stack_history() {
    let (ok, stdout, stderr) = script(":precision 80\nln:invalid\n:quit\n");
    assert!(!ok);
    assert!(stderr.contains("unknown token"));
    assert_eq!(stdout.trim(), "precision: 80 bits");
}

#[test]
fn constants_and_real_functions() {
    let (ok, stdout, stderr) = script(
        ":precision 80\n0 exp\nclear 1 ln\nclear 1 log10\nclear 0 sin\nclear 0 cos\nclear 0 tan\nclear 180 rad\n:undo\nclear pi\n:settings\n",
    );
    assert!(ok, "{stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        &lines[..7],
        ["precision: 80 bits", "1", "0", "0", "0", "1", "0"]
    );
    assert_eq!(lines[7], lines[9]); // 180 rad == pi at the same output precision.
    assert_eq!(lines[8], "0"); // Undo the 180->radian conversion.
    assert_eq!(
        lines[10],
        "mode=exact precision=80 bits rounding=nearest display=auto"
    );
}

#[test]
fn invalid_log_domain_does_not_modify_stack() {
    let (ok, stdout, stderr) = script("5\n0 ln\n:stack\n:undo\n:stack\n");
    assert!(!ok);
    assert!(stderr.contains("positive argument"));
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines, ["5", "[0] 5", "(empty)", "(empty)"]);
}

#[test]
fn finite_precision_limits_report_errors() {
    let (ok, _stdout, stderr) = script(":precision 513\n1 exp\n");
    assert!(!ok);
    assert!(stderr.contains("transcendental precision must be"));
}
