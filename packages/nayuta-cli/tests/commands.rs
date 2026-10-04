use std::{
    path::Path,
    process::{Command, Stdio},
};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nayuta"))
        .current_dir(cwd)
        .args(args)
        .env("NAYUTA_LOCALE", "en")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn help_bad_arguments_modes_and_non_tty_never_open_tui() {
    let empty = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let normal = run(empty.path(), &["--help"]);
    assert!(normal.status.success());
    let text = String::from_utf8_lossy(&normal.stdout);
    assert!(text.contains("create") && !text.contains("post new"));
    let ready = run(&root, &["--help"]);
    assert!(ready.status.success());
    let text = String::from_utf8_lossy(&ready.stdout);
    assert!(text.contains("post") && !text.contains("create"));
    for (cwd, args) in [
        (empty.path(), vec!["--bogus"]),
        (empty.path(), vec!["post", "new", "x"]),
        (&root, vec!["create", "x"]),
        (empty.path(), vec![]),
        (empty.path(), vec!["--tui"]),
    ] {
        let output = run(cwd, &args);
        assert_eq!(output.status.code(), Some(2));
        assert!(!output.stdout.windows(4).any(|w| w == b"\x1b[?1"));
        assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    }
    let version = run(empty.path(), &["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        format!("nayuta {}", env!("CARGO_PKG_VERSION"))
    );
}
