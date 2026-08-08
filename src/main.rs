use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_LIMIT: usize = 20;

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
}

struct Match {
    tool: Tool,
    score: i64,
}

fn print_usage() {
    println!(
        "ttf - terminal tool finder: fuzzy search tools.json\n\
         \n\
         Usage:\n  \
         \x20 ttf [OPTIONS] <query>    fuzzy search tools\n  \
         \x20 ttf [OPTIONS] --list     list all tools\n\
         \n\
         Options:\n  \
         \x20 -d, --data <path>   path to tools.json (default: exe dir or ./tools.json)\n  \
         \x20 -n, --limit <n>     max results (default: {DEFAULT_LIMIT})\n  \
         \x20 -l, --list          list all tools\n  \
         \x20 -h, --help          show this help"
    );
}

fn load_tools(path: &Path) -> Result<Vec<Tool>, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("invalid JSON in {}: {e}", path.display()))
}

fn find_data_path(explicit: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = explicit {
        return Some(PathBuf::from(p));
    }
    if let (Ok(exe), Some(dir)) = (env::current_exe(), None::<&Path>) {
        let _ = (exe, dir);
    }
    if let Ok(exe) = env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let cand = dir.join("tools.json");
        if cand.is_file() {
            return Some(cand);
        }
    }
    let cand = PathBuf::from("tools.json");
    cand.is_file().then_some(cand)
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

fn print_list(tools: &[Tool]) {
    let width = tools.iter().map(|t| t.name.len()).max().unwrap_or(0);
    for t in tools {
        print_tool(t, width, None);
    }
}

fn print_tool(t: &Tool, width: usize, score: Option<i64>) {
    let tagline = if t.tags.is_empty() {
        String::new()
    } else {
        format!("  [{}]", t.tags.join(", "))
    };
    match score {
        Some(s) => println!("{:>3}  {:<width$}  {}{}", s, t.name, t.description, tagline),
        None => println!("{:<width$}  {}{}", t.name, t.description, tagline),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return ExitCode::SUCCESS;
    }

    let mut data: Option<String> = None;
    let mut limit = DEFAULT_LIMIT;
    let mut list_all = false;
    let mut query_parts: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--data" => {
                i += 1;
                match args.get(i) {
                    Some(p) => data = Some(p.clone()),
                    None => {
                        eprintln!("error: {} requires a path", args[i - 1]);
                        return ExitCode::from(2);
                    }
                }
            }
            "-n" | "--limit" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse().ok()) {
                    Some(n) if n > 0 => limit = n,
                    _ => {
                        eprintln!("error: --limit requires a positive number");
                        return ExitCode::from(2);
                    }
                }
            }
            "-l" | "--list" => list_all = true,
            s if s.starts_with('-') && s.len() > 1 => {
                eprintln!("error: unknown option: {s}");
                return ExitCode::from(2);
            }
            s => query_parts.push(s.to_string()),
        }
        i += 1;
    }

    let Some(data_path) = find_data_path(data.as_deref()) else {
        eprintln!("error: tools.json not found (use -d <path>)");
        return ExitCode::from(2);
    };
    let tools = match load_tools(&data_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };

    if list_all {
        print_list(&tools);
        return ExitCode::SUCCESS;
    }

    let query = query_parts.join(" ");
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
        println!("no match for '{query}'");
        return ExitCode::from(1);
    }

    let shown = matches.len().min(limit);
    let width = matches[..shown]
        .iter()
        .map(|m| m.tool.name.len())
        .max()
        .unwrap_or(0);
    for m in &matches[..shown] {
        print_tool(&m.tool, width, Some(m.score));
    }
    if matches.len() > shown {
        println!("... and {} more (--limit to change)", matches.len() - shown);
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
}
