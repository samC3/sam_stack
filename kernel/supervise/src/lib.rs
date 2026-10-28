use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

pub type Result<T> = std::result::Result<T, String>;

pub fn herdr(args: &[&str]) -> Result<Value> {
    let out = Command::new("herdr")
        .args(args)
        .output()
        .map_err(|e| format!("herdr: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let why = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(format!("herdr {} failed: {why}", args[..2.min(args.len())].join(" ")));
    }
    let parsed: Value = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    Ok(parsed["result"].clone())
}

pub fn meta(folder: &Path) -> Result<Value> {
    let text = fs::read_to_string(folder.join("meta.json"))
        .map_err(|_| format!("{} has no meta.json; start it first", folder.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}/meta.json: {e}", folder.display()))
}

pub fn state(name: &str) -> String {
    herdr(&["agent", "get", name]).map_or_else(
        |_| "gone".to_string(),
        |r| r["agent"]["agent_status"].as_str().unwrap_or("unknown").to_string(),
    )
}

pub fn returned(folder: &Path) -> String {
    let (path, sent) = (folder.join("return.md"), folder.join("sent.md"));
    let Ok(text) = fs::read_to_string(&path) else {
        return "-".into();
    };
    let status = text
        .lines()
        .find_map(|l| {
            l.split_once(':')
                .filter(|(k, _)| k.eq_ignore_ascii_case("status"))
                .map(|(_, v)| v.trim())
        })
        .unwrap_or("?");
    let modified = |p: &Path| fs::metadata(p).and_then(|m| m.modified()).ok();
    match (modified(&path), modified(&sent)) {
        (Some(ret), Some(msg)) if ret < msg => format!("stale ({status})"),
        _ => status.to_string(),
    }
}

pub struct Session {
    pub folder: PathBuf,
    pub children: Vec<Session>,
}

impl Session {
    pub fn flatten(&self) -> Vec<&Session> {
        let mut all = vec![self];
        for child in &self.children {
            all.extend(child.flatten());
        }
        all
    }
}

pub fn tree(dir: &Path) -> Vec<Session> {
    if dir.join("meta.json").is_file() {
        return vec![Session {
            folder: dir.to_path_buf(),
            children: tree(&dir.join("sessions")),
        }];
    }
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    entries.sort();
    entries.iter().flat_map(|p| tree(p)).collect()
}
