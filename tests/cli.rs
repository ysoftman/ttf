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

fn run_in_dir(dir: &std::path::Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_ttf"))
        .args(args)
        .current_dir(dir)
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
    let lines: Vec<&str> = out
        .lines()
        .filter(|l| !l.trim_start().starts_with("https://"))
        .collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[3].starts_with("... and"));
}

#[test]
fn list_shows_all_without_limit() {
    let (out, code) = run(&["--nocolor", "-d", "tools.json", "-l"]);
    assert_eq!(code, 0);
    assert!(out.lines().count() > 100);
    assert!(
        out.lines()
            .any(|l| l.split_whitespace().any(|w| w == "ttf"))
    );
}

#[test]
fn search_respects_limit() {
    let (out, code) = run(&["--nocolor", "-d", "tools.json", "-n", "1", "디스크"]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = out
        .lines()
        .filter(|l| !l.trim_start().starts_with("https://"))
        .collect();
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
fn uses_embedded_data_without_local_file() {
    let dir = std::env::temp_dir().join(format!("ttf-cli-embedded-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (out, code) = run_in_dir(&dir, &["--nocolor", "ls"]);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(code, 0);
    assert!(out.contains("list directory contents"));
}

#[test]
fn search_shows_installed_marker() {
    let (out, code) = run(&["--nocolor", "-d", "tools.json", "-n", "1", "ls directory"]);
    assert_eq!(code, 0);
    assert!(out.contains('✓'));
    assert!(out.contains("list directory contents"));
}

#[test]
fn search_shows_url() {
    let (out, code) = run(&["--nocolor", "-d", "tools.json", "-n", "1", "ls directory"]);
    assert_eq!(code, 0);
    assert!(out.contains("https://github.com/coreutils/coreutils"));
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
    let (out, _) = run(&["--nocolor", "-d", "tools.json", "-n", "1", "ls directory"]);
    assert!(!out.contains('\x1b'));
    assert!(out.contains("list directory contents"));
}
