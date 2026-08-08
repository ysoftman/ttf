use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const DEFAULT_LIMIT: usize = 20;

const EMBEDDED_TOOLS: &str = include_str!("../tools.json");

macro_rules! print_line {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

const C_RESET: &str = "\x1b[0m";
const C_NAME: &str = "\x1b[32m";
const C_TAG: &str = "\x1b[36m";
const C_INSTALLED: &str = "\x1b[32m";
const C_NOT_INSTALLED: &str = "\x1b[31m";

#[derive(Debug, Clone, Deserialize)]
struct Tool {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    tags: Vec<String>,
}

impl Tool {
    fn searchable(&self) -> String {
        let mut s = self.name.clone();
        if !self.description.is_empty() {
            s.push(' ');
            s.push_str(&self.description);
        }
        if !self.tags.is_empty() {
            s.push(' ');
            s.push_str(&self.tags.join(" "));
        }
        s
    }

    fn is_builtin(&self) -> bool {
        self.tags.iter().any(|t| t == "built-in")
    }
}

struct Match {
    tool: Tool,
    score: i64,
}

fn print_usage() {
    print_line!(
        "ttf - terminal tool finder: fuzzy search tools.json\n\
         \n\
         Usage:\n  \
         \x20 ttf [OPTIONS] <query>    fuzzy search tools\n  \
         \n\
         Options:\n  \
         \x20 -d, --data <path>   path to external tools.json (default: embedded data)\n  \
         \x20 -n, --limit <n>     max results (default: {DEFAULT_LIMIT})\n  \
         \x20 -l, --list          list all tools\n  \
         \x20 -v, --version       show version\n  \
         \x20 --color             enable colors (default)\n  \
         \x20 --nocolor           disable colors\n  \
         \x20 -h, --help          show this help"
    );
}

fn load_tools(path: Option<&Path>) -> Result<Vec<Tool>, String> {
    match path {
        Some(p) => {
            let text =
                fs::read_to_string(p).map_err(|e| format!("cannot read {}: {e}", p.display()))?;
            serde_json::from_str(&text).map_err(|e| format!("invalid JSON in {}: {e}", p.display()))
        }
        None => serde_json::from_str(EMBEDDED_TOOLS)
            .map_err(|e| format!("invalid embedded tools.json: {e}")),
    }
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.is_file()
            && path
                .metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn is_installed(name: &str) -> bool {
    if name.contains('/') {
        return is_executable(Path::new(name));
    }
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| is_executable(&dir.join(name)))
    })
}

fn fuzzy_score(query: &str, target: &str) -> Option<i64> {
    let q: Vec<char> = query.to_lowercase().chars().collect();
    let t: Vec<char> = target.to_lowercase().chars().collect();
    if q.is_empty() {
        return Some(0);
    }
    let mut qi = 0usize;
    let mut score = 0i64;
    let mut prev: Option<usize> = None;
    for (ti, &c) in t.iter().enumerate() {
        if qi < q.len() && c == q[qi] {
            let mut s = 0i64;
            if ti == 0 {
                s += 32;
            }
            if let Some(p) = prev
                && ti == p + 1
            {
                s += 8;
            }
            if ti > 0 {
                let prev_char = t[ti - 1];
                if prev_char.is_whitespace()
                    || matches!(prev_char, '-' | '_' | '.' | '/' | ':' | '(')
                {
                    s += 16;
                }
            }
            score += s;
            prev = Some(ti);
            qi += 1;
        }
    }
    (qi == q.len()).then_some(score)
}

fn print_list(tools: &[Tool], limit: Option<usize>, color: bool) {
    let shown = limit.map(|l| tools.len().min(l)).unwrap_or(tools.len());
    let width = tools.iter().map(|t| t.name.len()).max().unwrap_or(0);
    for t in &tools[..shown] {
        print_tool(t, width, None, color);
    }
    if tools.len() > shown {
        print_line!("... and {} more", tools.len() - shown);
    }
}

fn print_tool(t: &Tool, width: usize, score: Option<i64>, color: bool) {
    let name_padded = format!("{:<width$}", t.name);
    let name = if color {
        format!("{C_NAME}{name_padded}{C_RESET}")
    } else {
        name_padded
    };
    let mark = if t.is_builtin() || is_installed(&t.name) {
        if color {
            format!("{C_INSTALLED}✓{C_RESET}")
        } else {
            "✓".to_string()
        }
    } else if color {
        format!("{C_NOT_INSTALLED}✗{C_RESET}")
    } else {
        "✗".to_string()
    };
    let tagline = if t.tags.is_empty() {
        String::new()
    } else if color {
        let tags: Vec<String> = t
            .tags
            .iter()
            .map(|tag| format!("{C_TAG}{tag}{C_RESET}"))
            .collect();
        format!("  [{}]", tags.join(", "))
    } else {
        format!("  [{}]", t.tags.join(", "))
    };
    match score {
        Some(s) => print_line!("{:>3}  {}  {}  {}{}", s, mark, name, t.description, tagline),
        None => print_line!("{}  {}  {}{}", mark, name, t.description, tagline),
    }
}

#[derive(Debug, PartialEq)]
struct Config {
    data: Option<String>,
    limit: usize,
    limit_explicit: bool,
    list_all: bool,
    query_parts: Vec<String>,
    help: bool,
    version: bool,
    color: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data: None,
            limit: DEFAULT_LIMIT,
            limit_explicit: false,
            list_all: false,
            query_parts: Vec::new(),
            help: false,
            version: false,
            color: true,
        }
    }
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        return Ok(Config {
            help: true,
            ..Config::default()
        });
    }

    let mut cfg = Config::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--data" => {
                i += 1;
                match args.get(i) {
                    Some(p) => cfg.data = Some(p.clone()),
                    None => return Err(format!("{} requires a path", args[i - 1])),
                }
            }
            "-n" | "--limit" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse().ok()) {
                    Some(n) if n > 0 => {
                        cfg.limit = n;
                        cfg.limit_explicit = true;
                    }
                    _ => return Err("--limit requires a positive number".to_string()),
                }
            }
            "-l" | "--list" => cfg.list_all = true,
            "-v" | "--version" => cfg.version = true,
            "--color" => cfg.color = true,
            "--nocolor" => cfg.color = false,
            s if s.starts_with('-') && s.len() > 1 => {
                return Err(format!("unknown option: {s}"));
            }
            s => cfg.query_parts.push(s.to_string()),
        }
        i += 1;
    }
    Ok(cfg)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = match parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    if cfg.help {
        print_usage();
        return ExitCode::SUCCESS;
    }
    if cfg.version {
        print_line!("ttf {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    let tools = match load_tools(cfg.data.as_deref().map(Path::new)) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };

    if cfg.list_all {
        print_list(&tools, cfg.limit_explicit.then_some(cfg.limit), cfg.color);
        return ExitCode::SUCCESS;
    }

    let query = cfg.query_parts.join(" ");
    if query.is_empty() {
        eprintln!("error: missing query (see --help)");
        return ExitCode::from(2);
    }

    let mut matches: Vec<Match> = tools
        .into_iter()
        .filter_map(|tool| {
            fuzzy_score(&query, &tool.searchable()).map(|score| Match { tool, score })
        })
        .collect();

    matches.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.tool.name.cmp(&b.tool.name))
    });

    if matches.is_empty() {
        print_line!("no match for '{query}'");
        return ExitCode::from(1);
    }

    let shown = matches.len().min(cfg.limit);
    let width = matches[..shown]
        .iter()
        .map(|m| m.tool.name.len())
        .max()
        .unwrap_or(0);
    for m in &matches[..shown] {
        print_tool(&m.tool, width, Some(m.score), cfg.color);
    }
    if matches.len() > shown {
        print_line!("... and {} more (--limit to change)", matches.len() - shown);
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
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
        let cfg =
            parse_args(&[arg("-d"), arg("data.json"), arg("-n"), arg("5"), arg("ls")]).unwrap();
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
        };
        assert!(t.is_builtin());
        let plain = Tool {
            name: "history".to_string(),
            description: String::new(),
            tags: Vec::new(),
        };
        assert!(!plain.is_builtin());
    }
}
