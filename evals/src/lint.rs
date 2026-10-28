use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::comments::{Lang, comment_lines};

const CONTRACT: [&str; 5] = ["Layer", "Input", "Output", "Stop", "Returns"];
const SKIPPED_DIRS: [&str; 7] = [
    ".git",
    ".claude",
    "target",
    "mutants.out",
    "mutants.out.old",
    "fixture",
    "work",
];

pub fn fields<'a>(lines: impl IntoIterator<Item = &'a str>) -> BTreeMap<String, String> {
    lines
        .into_iter()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect()
}

pub fn frontmatter(text: &str) -> Option<BTreeMap<String, String>> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first() != Some(&"---") {
        return None;
    }
    let end = lines[1..].iter().position(|l| *l == "---")? + 1;
    Some(fields(lines[1..end].iter().copied()))
}

fn block<'a>(text: &'a str, info: &str) -> Option<Vec<&'a str>> {
    let mut lines = text.lines().skip_while(|l| *l != format!("```{info}"));
    lines.next()?;
    let mut body = Vec::new();
    for line in lines {
        if line.starts_with("```") {
            return Some(body);
        }
        body.push(line);
    }
    None
}

fn layer_dir(layer: &str) -> Option<&'static str> {
    match layer {
        "verb" | "combinator" => Some("kernel"),
        "workflow" => Some("workflows"),
        _ => None,
    }
}

fn names_repo_relative_work(text: &str) -> bool {
    text.match_indices("work/").any(|(at, _)| {
        text[..at]
            .chars()
            .next_back()
            .is_none_or(|c| c == '\n' || !(c.is_alphanumeric() || "_/.$-".contains(c)))
    })
}

fn checks_invoked(checks: &str, name: &str) -> bool {
    let call = format!("invoked {name}");
    checks.lines().any(|line| {
        let line = line.trim_end_matches([' ', '\t']);
        line.strip_suffix(&call).is_some_and(|before| {
            before
                .chars()
                .next_back()
                .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
        })
    })
}

fn dirs(path: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    found.sort();
    found
}

fn code_files(dir: &Path, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if path.is_dir() && !SKIPPED_DIRS.contains(&name) {
            code_files(&path, found);
        } else if Lang::of(&path).is_some() {
            found.push(path);
        }
    }
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

struct Errors<'a> {
    root: &'a Path,
    found: Vec<String>,
}

impl Errors<'_> {
    fn add(&mut self, path: &Path, msg: impl Into<String>) {
        let shown = path.strip_prefix(self.root).unwrap_or(path);
        self.found.push(format!("{}: {}", shown.display(), msg.into()));
    }
}

fn dir_name(path: &Path) -> String {
    path.file_name().unwrap_or_default().to_string_lossy().into_owned()
}

fn skill_name(skill: &Path) -> String {
    dir_name(skill.parent().unwrap_or(skill))
}

fn check_skill(errors: &mut Errors, path: &Path) {
    let text = read(path);
    let dir = path.parent().unwrap_or(path);
    let (name, folder) = (dir_name(dir), dir_name(dir.parent().unwrap_or(dir)));
    let front = frontmatter(&text).unwrap_or_default();
    if front.get("name") != Some(&name) {
        errors.add(path, format!("frontmatter name must be '{name}', its directory"));
    }
    if front.get("description").is_none_or(String::is_empty) {
        errors.add(path, "frontmatter has no description");
    }
    let lines = block(&text, "contract");
    if lines.is_none() {
        errors.add(path, "no ```contract block");
    }
    let contract = fields(lines.unwrap_or_default());
    for key in CONTRACT {
        if contract.get(key).is_none_or(String::is_empty) {
            errors.add(path, format!("contract field '{key}' is missing or empty"));
        }
    }
    if let Some(layer) = contract.get("Layer").filter(|l| !l.is_empty()) {
        match layer_dir(layer) {
            None => errors.add(path, format!("Layer '{layer}' must be verb, combinator or workflow")),
            Some(want) if want != folder => errors.add(path, format!("a {layer} belongs in {want}/, not {folder}/")),
            Some(_) => {}
        }
    }
    if names_repo_relative_work(&text) {
        errors.add(
            path,
            "names a repo-relative work/ path; task state lives under $WORK/ (KERNEL.md, State)",
        );
    }
}

fn check_registry(errors: &mut Errors, kernel: &Path, names: &BTreeSet<String>) {
    let text = read(kernel);
    let rows = block(&text, "registry");
    if rows.is_none() {
        errors.add(kernel, "no ```registry block");
    }
    for row in rows.unwrap_or_default() {
        let cols: Vec<&str> = row.split_whitespace().collect();
        if cols.first().is_some_and(|c| !c.starts_with('#')) && cols.get(1).is_none_or(|w| !names.contains(*w)) {
            errors.add(
                kernel,
                format!("registry row '{}' needs a writer that is a skill", row.trim()),
            );
        }
    }
}

fn registered_files(kernel: &Path) -> BTreeSet<String> {
    block(&read(kernel), "registry")
        .unwrap_or_default()
        .iter()
        .filter_map(|row| row.split_whitespace().next())
        .map(String::from)
        .collect()
}

fn shared_paths(text: &str) -> Vec<&str> {
    text.match_indices("$SAM_STACK_WORK/")
        .map(|(at, _)| {
            text[at..]
                .split(|c: char| c.is_whitespace() || c == '`')
                .next()
                .unwrap_or_default()
                .trim_end_matches([',', '.', ';', ')'])
        })
        .collect()
}

fn check_shared_outputs(errors: &mut Errors, skill: &Path, registered: &BTreeSet<String>) {
    let contract = fields(block(&read(skill), "contract").unwrap_or_default());
    for path in shared_paths(contract.get("Output").map_or("", String::as_str)) {
        if !registered.contains(path) {
            errors.add(
                skill,
                format!("writes {path}, which KERNEL.md's registry doesn't list (KERNEL.md, State)"),
            );
        }
    }
}

fn check_task(errors: &mut Errors, task: &Path, names: &BTreeSet<String>) -> Vec<String> {
    let front = frontmatter(&read(&task.join("task.md"))).unwrap_or_default();
    let covers: Vec<String> = front
        .get("covers")
        .map(|c| {
            c.split(',')
                .map(str::trim)
                .filter(|c| !c.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    if covers.is_empty() {
        errors.add(
            task,
            "task.md needs frontmatter naming the skills it covers (covers: a, b)",
        );
    }
    let has_checks = task.join("checks.sh").is_file();
    let checks = read(&task.join("checks.sh"));
    for name in &covers {
        if !names.contains(name) {
            errors.add(task, format!("covers '{name}', which isn't a skill"));
        }
        if !checks_invoked(&checks, name) {
            errors.add(
                task,
                format!("covers '{name}' but checks.sh has no `invoked {name}` check"),
            );
        }
    }
    let turns = front.get("turns").map_or("1", String::as_str);
    if turns.is_empty() || !turns.chars().all(|c| c.is_ascii_digit()) {
        errors.add(task, "turns: must be a whole number");
    }
    if !task.join("prompt.md").is_file() {
        errors.add(task, "no prompt.md");
    }
    if !has_checks && !task.join("rubric.md").is_file() {
        errors.add(task, "needs checks.sh or rubric.md");
    }
    for reference in ["apply.sh", "transcript.jsonl"] {
        if has_checks && !task.join("reference").join(reference).is_file() {
            errors.add(
                task,
                format!("checks.sh has no reference/{reference} to test it against (evals graders)"),
            );
        }
    }
    covers
}

fn check_code(errors: &mut Errors, root: &Path) {
    let mut code = Vec::new();
    code_files(root, &mut code);
    for path in code {
        let text = read(&path);
        for line in Lang::of(&path)
            .map(|lang| comment_lines(lang, &text))
            .unwrap_or_default()
        {
            errors.add(&path, format!("line {line} is a comment; code has none (STANDARDS.md)"));
        }
    }
}

pub fn lint(root: &Path) -> Vec<String> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut errors = Errors {
        root: &root,
        found: Vec::new(),
    };
    let skills: Vec<PathBuf> = ["kernel", "workflows"]
        .iter()
        .flat_map(|layer| dirs(&root.join(layer)))
        .map(|d| d.join("SKILL.md"))
        .filter(|p| p.is_file())
        .collect();
    let mut names: BTreeSet<String> = skills.iter().map(|p| skill_name(p)).collect();
    let installed = fs::read_dir(root.join(".claude/skills"))
        .into_iter()
        .flatten()
        .flatten();
    names.extend(installed.map(|e| dir_name(&e.path())));

    for path in &skills {
        check_skill(&mut errors, path);
    }
    let kernel = root.join("kernel/KERNEL.md");
    if kernel.is_file() {
        check_registry(&mut errors, &kernel, &names);
        let registered = registered_files(&kernel);
        for path in &skills {
            check_shared_outputs(&mut errors, path, &registered);
        }
    }
    let mut covered = BTreeSet::new();
    for task in dirs(&root.join("evals/tasks")) {
        covered.extend(check_task(&mut errors, &task, &names));
    }
    for path in &skills {
        if !covered.contains(&skill_name(path)) {
            errors.add(
                path,
                "no eval task covers this skill (covers: in evals/tasks/*/task.md)",
            );
        }
    }
    check_code(&mut errors, &root);
    errors.found
}

#[cfg(test)]
mod tests;
