use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static ID: AtomicUsize = AtomicUsize::new(0);

fn directory() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "rpning-programming-{}-{}",
        std::process::id(),
        ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn script_definitions_comments_control_flow_and_nonzero_errors() {
    let directory = directory();
    let path = directory.join("invoice.rpn");
    std::fs::write(
        &path,
        "# Exact invoice calculation\n: vat \"27 percent\" 1.27 * ;\n10000 vat\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .arg("--run")
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "12700");
    std::fs::write(&path, "1 0 /").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rpn"))
        .arg("--run")
        .arg(&path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn macros_persist_across_processes_and_imports_cannot_execute_code() {
    let directory = directory();
    let run = |source: &str| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rpn"))
            .env("RPNING_HOME", &directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(source.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    };
    let saved = run(": square dup * ;\n:alias sq square\n:macros-save\n");
    assert!(saved.status.success(), "{:?}", saved.stderr);
    let loaded = run(":macros-load\n7 sq\n:see square\n:trace 2 square\n");
    assert!(loaded.status.success(), "{:?}", loaded.stderr);
    let stdout = String::from_utf8_lossy(&loaded.stdout);
    assert!(stdout.lines().any(|line| line == "49"));
    assert!(stdout.contains("depth=1"));
    std::fs::write(directory.join("macros.rpn"), ": valid 1 ; 99 sto:oops").unwrap();
    let invalid = run("7\n:macros-load\n:macros\n:stack\n:registers\n");
    assert!(!invalid.status.success());
    let stdout = String::from_utf8_lossy(&invalid.stdout);
    assert!(stdout.contains("(no macros)"));
    assert!(stdout.contains("[0] 7"));
    assert!(stdout.contains("(no registers)"));
    std::fs::remove_file(directory.join("macros.rpn")).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
