use super::*;

#[test]
fn subsequence_required() {
    assert!(fuzzy_score("grep", "grep").is_some());
    assert!(fuzzy_score("grep", "find").is_none());
    assert!(fuzzy_score("gzz", "grep").is_none());
}

#[test]
fn order_matters() {
    assert!(fuzzy_score("df", "find").is_none());
    assert!(fuzzy_score("fd", "find").is_some());
}

#[test]
fn case_insensitive() {
    assert_eq!(fuzzy_score("GIT", "git"), fuzzy_score("git", "git"));
}

#[test]
fn prefix_scores_higher() {
    assert!(fuzzy_score("ls", "ls").unwrap() > fuzzy_score("ls", "closet").unwrap());
}

#[test]
fn matches_tags() {
    assert!(fuzzy_score("검색", "grep 검색 패턴 regex 정규식").is_some());
}

fn arg(s: &str) -> String {
    s.to_string()
}

static TEMP_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn temp_dir() -> std::path::PathBuf {
    let n = TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("ttf-test-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn parse_args_defaults() {
    let cfg = parse_args(&[arg("ls")]).unwrap();
    assert_eq!(cfg.data, None);
    assert_eq!(cfg.limit, DEFAULT_LIMIT);
    assert!(!cfg.list_all);
    assert!(!cfg.help);
    assert!(!cfg.version);
    assert_eq!(cfg.query_parts, vec!["ls"]);
}

#[test]
fn parse_args_multiple_query_parts() {
    let cfg = parse_args(&[arg("디렉토리"), arg("이동")]).unwrap();
    assert_eq!(cfg.query_parts, vec!["디렉토리", "이동"]);
}

#[test]
fn parse_args_data_and_limit() {
    let cfg = parse_args(&[arg("-d"), arg("data.json"), arg("-n"), arg("5"), arg("ls")]).unwrap();
    assert_eq!(cfg.data.as_deref(), Some("data.json"));
    assert_eq!(cfg.limit, 5);
}

#[test]
fn parse_args_long_flags() {
    let cfg = parse_args(&[
        arg("--data"),
        arg("x.json"),
        arg("--limit"),
        arg("10"),
        arg("--list"),
    ])
    .unwrap();
    assert_eq!(cfg.data.as_deref(), Some("x.json"));
    assert_eq!(cfg.limit, 10);
    assert!(cfg.list_all);
}

#[test]
fn parse_args_version() {
    let cfg = parse_args(&[arg("--version")]).unwrap();
    assert!(cfg.version);
}

#[test]
fn parse_args_color_default_on() {
    let cfg = parse_args(&[arg("ls")]).unwrap();
    assert!(cfg.color);
}

#[test]
fn parse_args_nocolor() {
    let cfg = parse_args(&[arg("--nocolor")]).unwrap();
    assert!(!cfg.color);
}

#[test]
fn parse_args_color_re_enable() {
    let cfg = parse_args(&[arg("--nocolor"), arg("--color")]).unwrap();
    assert!(cfg.color);
}

#[test]
fn parse_args_help_wins() {
    let cfg = parse_args(&[arg("ls"), arg("-h"), arg("-z")]).unwrap();
    assert!(cfg.help);
    assert!(!cfg.version);
}

#[test]
fn parse_args_unknown_option() {
    assert_eq!(
        parse_args(&[arg("-z")]),
        Err("unknown option: -z".to_string())
    );
}

#[test]
fn parse_args_missing_data_value() {
    assert_eq!(
        parse_args(&[arg("-d")]),
        Err("-d requires a path".to_string())
    );
}

#[test]
fn parse_args_missing_limit_value() {
    assert_eq!(
        parse_args(&[arg("-n")]),
        Err("--limit requires a positive number".to_string())
    );
}

#[test]
fn parse_args_zero_limit_rejected() {
    assert_eq!(
        parse_args(&[arg("-n"), arg("0")]),
        Err("--limit requires a positive number".to_string())
    );
}

#[test]
fn load_tools_valid_json() {
    let dir = temp_dir();
    let path = dir.join("tools.json");
    std::fs::write(
        &path,
        r#"[{"name":"ls","description":"list","tags":["디렉토리"]}]"#,
    )
    .unwrap();

    let tools = load_tools(Some(&path)).unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "ls");
    assert_eq!(tools[0].description, "list");
    assert_eq!(tools[0].tags, vec!["디렉토리"]);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_tools_missing_file() {
    let path = std::env::temp_dir().join("ttf-does-not-exist.json");
    assert!(load_tools(Some(&path)).is_err());
}

#[test]
fn load_tools_invalid_json() {
    let dir = temp_dir();
    let path = dir.join("tools.json");
    std::fs::write(&path, "{not json}").unwrap();

    assert!(load_tools(Some(&path)).is_err());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_tools_embedded() {
    let tools = load_tools(None).unwrap();
    assert!(tools.len() > 100);
    assert!(tools.iter().any(|t| t.name == "ttf"));
}

#[test]
fn load_tools_missing_optional_fields() {
    let dir = temp_dir();
    let path = dir.join("tools.json");
    std::fs::write(&path, r#"[{"name":"ls"}]"#).unwrap();

    let tools = load_tools(Some(&path)).unwrap();
    assert_eq!(tools[0].description, "");
    assert!(tools[0].tags.is_empty());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn is_installed_finds_common_command() {
    assert!(is_installed("ls"));
}

#[test]
fn is_installed_missing_command() {
    assert!(!is_installed("definitely-not-a-real-command-xyz"));
}

#[test]
fn is_installed_handles_slash_path() {
    assert!(is_installed("/bin/ls"));
    assert!(!is_installed("/bin/definitely-not-real"));
}

#[test]
fn builtin_tag_marked_installed() {
    let t = Tool {
        name: "history".to_string(),
        description: String::new(),
        tags: vec!["built-in".to_string()],
        url: String::new(),
        lang: String::new(),
    };
    assert!(t.is_builtin());
    let plain = Tool {
        name: "history".to_string(),
        description: String::new(),
        tags: Vec::new(),
        url: String::new(),
        lang: String::new(),
    };
    assert!(!plain.is_builtin());
}
