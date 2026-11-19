use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn settings_shortcuts_bindings_and_history_roundtrip_between_processes() {
    let directory = std::env::temp_dir().join(format!("rpning-workflow-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
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
    let saved = run(
        ":precision 128\n:round down\n:display fixed:2\n:shortcut vat 1.27 *\n:bind nav home @shortcut:vat\n:settings-save\n100\n:do vat\n:history-save\n",
    );
    assert!(saved.status.success(), "{:?}", saved.stderr);
    assert!(
        String::from_utf8_lossy(&saved.stdout)
            .lines()
            .any(|line| line == "127.00")
    );
    let loaded = run(
        ":settings-load\n:settings\n:history-load\n:history vat\n:bindings\n:complete :mac\n1 3 /\n:display fraction\n:stack\n",
    );
    assert!(loaded.status.success(), "{:?}", loaded.stderr);
    let text = String::from_utf8_lossy(&loaded.stdout);
    for expected in [
        "precision=128 bits rounding=down display=fixed:2",
        "nav.home = @shortcut:vat",
        ":macros-save",
        "~0.33",
        "[0] 1/3",
        ":shortcut vat 1.27 *",
    ] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
    std::fs::write(
        directory.join("settings.txt"),
        "RPNING-CONFIG-1\nprecision\t99999\n",
    )
    .unwrap();
    let invalid = run(":precision 53\n:settings-load\n:settings\n");
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stdout).contains("precision=53 bits"));
    for name in ["settings.txt", "history.txt"] {
        std::fs::remove_file(directory.join(name)).unwrap();
    }
    std::fs::remove_dir(directory).unwrap();
}
