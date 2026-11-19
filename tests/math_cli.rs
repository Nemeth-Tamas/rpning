use std::io::Write;
use std::process::{Command, Stdio};

fn run_script(input: &str) -> (bool, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to launch rpn");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    (
        output.status.success(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

#[test]
fn exact_math_and_scientific_formatting() {
    let (ok, stdout, stderr) = run_script(
        "2 10 pow\nclear 81/16 sqrt\nclear -27 3 root\nclear 6 fact\nclear -48 18 gcd\nclear 12 18 lcm\n:display scientific\nclear 12345\n",
    );
    assert!(ok, "{stderr}");
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [
            "1024",
            "2.25",
            "-3",
            "720",
            "6",
            "36",
            "display: scientific",
            "1.2345e4"
        ]
    );
}

#[test]
fn binary_approximation_and_display_settings() {
    let (ok, stdout, stderr) = run_script(
        ":mode binary\n:precision 4\n1 3 /\n:round zero\nclear 1 3 /\n:mode exact\nclear 1 3 /\napprox\n:settings\n",
    );
    assert!(ok, "{stderr}");
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [
            "numeric mode: binary",
            "precision: 4 bits",
            "0.34375",
            "rounding: zero",
            "0.3125",
            "numeric mode: exact",
            "1/3",
            "0.3125",
            "mode=exact precision=4 bits rounding=zero display=auto",
        ]
    );
}

#[test]
fn math_errors_are_atomic_in_interactive_mode() {
    let (ok, stdout, stderr) = run_script("5\n2 sqrt\n:stack\n:undo\n:stack\n");
    // Interactive errors are intentionally nonfatal, unlike piped stdin.
    assert!(!ok);
    assert!(stderr.contains("root is not rational"));
    assert!(stdout.contains("[0] 5"));
}
