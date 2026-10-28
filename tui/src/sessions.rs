use std::fs;
use std::path::{Path, PathBuf};

use supervise::Session;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    Project(String),
    Slug(String),
    Session {
        depth: usize,
        name: String,
        state: String,
        returned: String,
        folder: PathBuf,
    },
}

fn dirs(dir: &Path, skip: &[&str]) -> Vec<(String, PathBuf)> {
    let mut found: Vec<(String, PathBuf)> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| Some((e.file_name().to_str()?.to_string(), e.path())))
        .filter(|(name, _)| !skip.contains(&name.as_str()))
        .collect();
    found.sort();
    found
}

fn push(rows: &mut Vec<Row>, session: &Session, depth: usize, state: &dyn Fn(&str) -> String) {
    let folder = &session.folder;
    let name = supervise::meta(folder)
        .ok()
        .and_then(|m| m["name"].as_str().map(str::to_string))
        .unwrap_or_else(|| folder.file_name().unwrap_or_default().to_string_lossy().into_owned());
    rows.push(Row::Session {
        depth,
        state: state(&name),
        name,
        returned: supervise::returned(folder),
        folder: folder.clone(),
    });
    for child in &session.children {
        push(rows, child, depth + 1, state);
    }
}

pub fn load(work: &Path, state: &dyn Fn(&str) -> String) -> Vec<Row> {
    let mut rows = Vec::new();
    for (project, path) in dirs(work, &["lessons", "todo"]) {
        let mut named = false;
        for (slug, path) in dirs(&path, &["_archive"]) {
            let sessions = supervise::tree(&path.join("sessions"));
            if sessions.is_empty() {
                continue;
            }
            if !named {
                rows.push(Row::Project(project.clone()));
                named = true;
            }
            rows.push(Row::Slug(slug));
            for session in &sessions {
                push(&mut rows, session, 0, state);
            }
        }
    }
    rows
}

#[cfg(test)]
mod tests;
