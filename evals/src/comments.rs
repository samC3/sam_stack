use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Shell,
    Python,
}

impl Lang {
    pub fn of(path: &Path) -> Option<Lang> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Some(Lang::Rust),
            Some("sh") => Some(Lang::Shell),
            Some("py") => Some(Lang::Python),
            None if path.parent().and_then(Path::file_name).is_some_and(|d| d == "bin") => Some(Lang::Shell),
            _ => None,
        }
    }
}

pub fn comment_lines(lang: Lang, src: &str) -> BTreeSet<usize> {
    match lang {
        Lang::Rust => rust(src),
        Lang::Shell => shell(src),
        Lang::Python => python(src),
    }
}

fn rust(src: &str) -> BTreeSet<usize> {
    let chars: Vec<char> = src.chars().collect();
    let mut found = BTreeSet::new();
    let mut line = 1;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            found.insert(line);
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && next == Some('*') {
            found.insert(line);
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    line += usize::from(chars[i] == '\n');
                    i += 1;
                }
            }
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if matches!(word.as_str(), "r" | "br" | "cr") && matches!(chars.get(i), Some('#' | '"')) {
                i = raw_string_end(&chars, i, &mut line);
            }
            continue;
        }
        match c {
            '"' => i = string_end(&chars, i + 1, &mut line),
            '\'' => i = char_literal_end(&chars, i),
            '\n' => {
                line += 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    found
}

fn string_end(chars: &[char], mut i: usize, line: &mut usize) -> usize {
    while i < chars.len() {
        match chars[i] {
            '\\' => i += 1,
            '"' => return i + 1,
            '\n' => *line += 1,
            _ => {}
        }
        i += 1;
    }
    i
}

fn raw_string_end(chars: &[char], mut i: usize, line: &mut usize) -> usize {
    let hashes = chars[i..].iter().take_while(|&&c| c == '#').count();
    i += hashes;
    if chars.get(i) != Some(&'"') {
        return i;
    }
    i += 1;
    while i < chars.len() {
        if chars[i] == '"' && chars[i + 1..].iter().take(hashes).filter(|&&c| c == '#').count() == hashes {
            return i + 1 + hashes;
        }
        *line += usize::from(chars[i] == '\n');
        i += 1;
    }
    i
}

fn char_literal_end(chars: &[char], i: usize) -> usize {
    match (chars.get(i + 1), chars.get(i + 2)) {
        (Some('\\'), _) => {
            let mut j = i + 3;
            while j < chars.len() && chars[j] != '\'' {
                j += 1;
            }
            j + 1
        }
        (Some(_), Some('\'')) => i + 3,
        _ => i + 1,
    }
}

fn shell(src: &str) -> BTreeSet<usize> {
    let mut found = BTreeSet::new();
    let mut heredoc: Option<String> = None;
    let mut quote: Option<char> = None;
    for (n, text) in src.lines().enumerate() {
        let line = n + 1;
        if let Some(end) = &heredoc {
            if text.trim_start_matches('\t') == end {
                heredoc = None;
            }
            continue;
        }
        if line == 1 && text.starts_with("#!") {
            continue;
        }
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            match quote {
                Some('\'') if c == '\'' => quote = None,
                Some('"') | None if c == '\\' => i += 1,
                Some('"') if c == '"' => quote = None,
                None if c == '\'' || c == '"' => quote = Some(c),
                None if c == '#' && (i == 0 || chars[i - 1].is_whitespace()) => {
                    let comment: String = chars[i..].iter().collect();
                    if !comment.starts_with("# shellcheck") {
                        found.insert(line);
                    }
                    break;
                }
                Some(_) | None => {}
            }
            i += 1;
        }
        if quote.is_none() {
            heredoc = heredoc_delimiter(text);
        }
    }
    found
}

fn heredoc_delimiter(text: &str) -> Option<String> {
    let at = text.find("<<")?;
    let rest = text[at + 2..].trim_start_matches(['-', '~']).trim_start();
    let rest = rest.trim_start_matches(['\'', '"']);
    let word: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    (!word.is_empty()).then_some(word)
}

fn python(src: &str) -> BTreeSet<usize> {
    let mut found = BTreeSet::new();
    let mut statement_starts_block = true;
    let mut open: Option<String> = None;
    for (n, text) in src.lines().enumerate() {
        let line = n + 1;
        let mut rest = text;
        if let Some(delim) = open.take() {
            let Some(at) = text.find(&delim) else {
                open = Some(delim);
                continue;
            };
            rest = &text[at + delim.len()..];
        } else if statement_starts_block && text.trim_start().starts_with(['"', '\'']) {
            found.insert(line);
        }
        let scan = python_line(rest);
        if let Some(comment) = scan.comment {
            let directive = (line == 1 && comment.starts_with("#!"))
                || ["noqa", "type:", "coding:", "coding="]
                    .iter()
                    .any(|d| comment.contains(d));
            if !directive {
                found.insert(line);
            }
        }
        if let Some(last) = scan.last_code {
            statement_starts_block = last == ':';
        }
        if scan.open.is_some() {
            statement_starts_block = false;
        }
        open = scan.open;
    }
    found
}

struct PythonLine<'a> {
    comment: Option<&'a str>,
    last_code: Option<char>,
    open: Option<String>,
}

fn python_line(text: &str) -> PythonLine<'_> {
    let mut last_code = None;
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        match c {
            '#' => {
                return PythonLine {
                    comment: Some(&text[at..]),
                    last_code,
                    open: None,
                };
            }
            '"' | '\'' => {
                let triple: String = std::iter::repeat_n(c, 3).collect();
                if text[at..].starts_with(&triple) {
                    let body = at + 3;
                    let Some(close) = text[body..].find(&triple) else {
                        return PythonLine {
                            comment: None,
                            last_code: Some(c),
                            open: Some(triple),
                        };
                    };
                    let end = body + close + 3;
                    while chars.peek().is_some_and(|&(i, _)| i < end) {
                        chars.next();
                    }
                } else {
                    while let Some((_, s)) = chars.next() {
                        if s == '\\' {
                            chars.next();
                        } else if s == c {
                            break;
                        }
                    }
                }
                last_code = Some(c);
            }
            _ if !c.is_whitespace() => last_code = Some(c),
            _ => {}
        }
    }
    PythonLine {
        comment: None,
        last_code,
        open: None,
    }
}

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn added_lines(diff: &str) -> BTreeMap<String, BTreeSet<usize>> {
    let mut added: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    let mut path = None;
    for line in diff.lines() {
        if let Some(target) = line.strip_prefix("+++ ") {
            path = target.strip_prefix("b/").map(String::from);
        }
        let Some(hunk) = line
            .strip_prefix("@@ -")
            .and_then(|h| h.split_once(" +"))
            .map(|(_, h)| h)
        else {
            continue;
        };
        let range = hunk.split_whitespace().next().unwrap_or_default();
        let (start, count) = range.split_once(',').unwrap_or((range, "1"));
        if let (Some(path), Ok(start), Ok(count)) = (&path, start.parse::<usize>(), count.parse::<usize>()) {
            added.entry(path.clone()).or_default().extend(start..start + count);
        }
    }
    added
}

pub fn added_since(repo: &Path, rev: &str) -> Result<Vec<String>, String> {
    let mut added = added_lines(&git(repo, &["diff", "-U0", rev])?);
    for file in git(repo, &["ls-files", "--others", "--exclude-standard"])?.lines() {
        let count = fs::read_to_string(repo.join(file))
            .map(|t| t.lines().count())
            .unwrap_or_default();
        added.entry(file.to_string()).or_default().extend(1..=count);
    }
    let mut found = Vec::new();
    for (file, lines) in added {
        let path = repo.join(&file);
        let (Some(lang), Ok(src)) = (Lang::of(&path), fs::read_to_string(&path)) else {
            continue;
        };
        for line in comment_lines(lang, &src).intersection(&lines) {
            found.push(format!(
                "{file}:{line} {}",
                src.lines().nth(line - 1).unwrap_or_default().trim()
            ));
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests;
