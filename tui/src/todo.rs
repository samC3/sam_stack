use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Todo,
    Done,
    Moved,
    Dropped,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub status: Status,
    pub text: String,
    pub project: Option<String>,
    pub slug: Option<String>,
    pub details: Option<PathBuf>,
    pub moved: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Todo {
    pub date: Option<String>,
    pub today: bool,
    pub items: Vec<Item>,
    pub backlog: Vec<Item>,
    pub now: Vec<String>,
}

fn status(word: &str) -> Option<Status> {
    match word {
        "TODO" => Some(Status::Todo),
        "DONE" => Some(Status::Done),
        "MOVED" => Some(Status::Moved),
        "DROPPED" => Some(Status::Dropped),
        "NOTE" => Some(Status::Note),
        _ => None,
    }
}

fn cut_moved(rest: &mut String) -> Option<String> {
    let at = rest.rfind(" (to ").filter(|_| rest.ends_with(')'))?;
    let to = rest[at + 5..rest.len() - 1].to_string();
    rest.truncate(at);
    Some(to)
}

fn cut_details(rest: &mut String, base: &Path) -> Option<PathBuf> {
    let at = rest.find("[details](")?;
    let tail = &rest[at + 10..];
    let end = tail.find(')')?;
    let path = base.join(&tail[..end]);
    *rest = format!("{}{}", &rest[..at], &tail[end + 1..]);
    Some(path)
}

fn cut_slug(rest: &mut String) -> Option<String> {
    let at = rest.find(" → ")?;
    let slug = rest[at + " → ".len()..].trim().to_string();
    rest.truncate(at);
    Some(slug)
}

fn cut_project(rest: &mut String) -> Option<String> {
    let trimmed = rest.trim_end();
    let at = trimmed.rfind("  [").filter(|_| trimmed.ends_with(']'))?;
    let project = trimmed[at + 3..trimmed.len() - 1].to_string();
    rest.truncate(at);
    Some(project)
}

pub fn parse(line: &str, base: &Path) -> Option<Item> {
    let (word, rest) = line.split_once(' ')?;
    let status = status(word)?;
    let mut rest = rest.trim().to_string();
    let moved = cut_moved(&mut rest);
    let details = cut_details(&mut rest, base);
    let slug = cut_slug(&mut rest);
    let project = cut_project(&mut rest);
    Some(Item {
        status,
        text: rest.trim().to_string(),
        project,
        slug,
        details,
        moved,
    })
}

fn items(path: &Path) -> Vec<Item> {
    let base = path.parent().unwrap_or(Path::new("."));
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| parse(l, base))
        .collect()
}

fn latest_day(days: &Path, today: &str) -> Option<String> {
    fs::read_dir(days)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().to_str()?.strip_suffix(".md").map(str::to_string))
        .filter(|d| d.as_str() <= today)
        .max()
}

pub fn load(dir: &Path, today: &str) -> Todo {
    let date = latest_day(&dir.join("days"), today);
    let now = fs::read_to_string(dir.join("now.md"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();
    Todo {
        today: date.as_deref() == Some(today),
        items: date
            .as_ref()
            .map(|d| items(&dir.join("days").join(format!("{d}.md"))))
            .unwrap_or_default(),
        date,
        backlog: items(&dir.join("backlog.md")),
        now,
    }
}

#[cfg(test)]
mod tests;
