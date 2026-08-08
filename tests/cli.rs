use std::process::Command;

fn run(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_ttf"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn list_respects_limit() {
    let (out, code) = run(&["--nocolor", "-d", "tools.json", "-l", "-n", "3"]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[3].starts_with("... and"));
}

#[test]
fn list_shows_all_without_limit() {
    let (out, code) = run(&["--nocolor", "-d", "tools.json", "-l"]);
    assert_eq!(code, 0);
    assert!(out.lines().count() > 100);
    assert!(out.lines().any(|l| l.starts_with("ttf")));
}

#[test]
fn search_respects_limit() {
    let (out, code) = run(&["--nocolor", "-d", "tools.json", "-n", "1", "디스크"]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[1].starts_with("... and"));
}

#[test]
fn version_prints_version() {
    let (out, code) = run(&["--version"]);
    assert_eq!(code, 0);
    assert!(out.trim().starts_with("ttf "));
}

#[test]
fn color_enabled_by_default() {
    let (out, _) = run(&["-d", "tools.json", "-n", "1", "ls"]);
    assert!(out.contains("\x1b[32m"));
    assert!(out.contains("\x1b[36m"));
    assert!(out.contains("\x1b[0m"));
}

#[test]
fn nocolor_omits_ansi() {
    let (out, _) = run(&["--nocolor", "-d", "tools.json", "-n", "1", "list"]);
    assert!(!out.contains('\x1b'));
    assert!(out.contains("list directory contents"));
}
