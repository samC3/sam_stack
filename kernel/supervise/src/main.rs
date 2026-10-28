use std::fmt::Write as _;
use std::fs;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::thread::sleep;
use std::time::Duration;

use clap::{Parser, Subcommand};
use serde_json::{Value, json};

#[derive(Parser)]
#[command(about = "Start, message, list and wait on supervised Claude Code sessions in herdr")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(about = "Start a session for <dir>; needs <dir>/brief.md")]
    Start {
        dir: PathBuf,
        #[arg(long, help = "give the session its own worktree and branch")]
        edit: bool,
        #[arg(long, help = "where the session starts (default: here)")]
        cwd: Option<PathBuf>,
    },
    #[command(about = "Send the session a message")]
    Send { dir: PathBuf, message: String },
    #[command(about = "Every session in or under <dir>, as a tree")]
    Status { dir: PathBuf },
    #[command(about = "Wait until one of the sessions is idle, done or blocked")]
    Wait {
        #[arg(required = true)]
        dirs: Vec<PathBuf>,
    },
}

type Result<T> = std::result::Result<T, String>;

fn herdr(args: &[&str]) -> Result<Value> {
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

fn agent_name(folder: &Path) -> Result<String> {
    let real = folder
        .canonicalize()
        .map_err(|e| format!("{}: {e}", folder.display()))?;
    let parts: Vec<String> = real.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let Some(first) = parts.iter().position(|p| p == "sessions") else {
        return Err(format!("{} is not under a sessions/ folder", folder.display()));
    };
    let name = parts[first..]
        .iter()
        .filter(|p| *p != "sessions")
        .cloned()
        .collect::<Vec<_>>()
        .join("-");
    let valid = name.len() <= 32
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    if !valid {
        return Err(format!(
            "herdr agent name '{name}' must match [a-z][a-z0-9_-]{{0,31}}; use shorter lowercase folder names"
        ));
    }
    Ok(name)
}

fn meta(folder: &Path) -> Result<Value> {
    let text = fs::read_to_string(folder.join("meta.json"))
        .map_err(|_| format!("{} has no meta.json; start it first", folder.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}/meta.json: {e}", folder.display()))
}

fn meta_name(folder: &Path) -> Result<String> {
    Ok(meta(folder)?["name"].as_str().unwrap_or_default().to_string())
}

fn now() -> String {
    let out = Command::new("date")
        .arg("+%Y-%m-%dT%H:%M:%S")
        .output()
        .map(|o| o.stdout)
        .unwrap_or_default();
    String::from_utf8_lossy(&out).trim().to_string()
}

fn uuid() -> Result<String> {
    let mut bytes = [0u8; 16];
    fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| e.to_string())?;
    bytes[6] = (bytes[6] & 0x0f) + 0x40;
    bytes[8] = (bytes[8] & 0x3f) + 0x80;
    let hex = bytes.iter().fold(String::new(), |mut hex, b| {
        let _ = write!(hex, "{b:02x}");
        hex
    });
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    ))
}

fn start(folder: &Path, edit: bool, cwd: &Path) -> Result<()> {
    if !folder.join("brief.md").is_file() {
        return Err(format!("write {}/brief.md first", folder.display()));
    }
    if folder.join("meta.json").exists() {
        return Err(format!("{} is already started; use send", folder.display()));
    }
    let name = agent_name(folder)?;
    let cwd = cwd.display().to_string();
    let branch = format!("supervise/{name}");
    let made = if edit {
        herdr(&[
            "worktree",
            "create",
            "--cwd",
            &cwd,
            "--branch",
            &branch,
            "--label",
            &name,
            "--no-focus",
        ])?
    } else {
        let workspace = std::env::var("HERDR_WORKSPACE_ID").unwrap_or_default();
        herdr(&[
            "tab",
            "create",
            "--workspace",
            &workspace,
            "--cwd",
            &cwd,
            "--label",
            &name,
            "--no-focus",
        ])?
    };
    let pane = made["root_pane"]["pane_id"]
        .as_str()
        .ok_or("herdr gave no root pane id")?
        .to_string();
    let session_id = uuid()?;
    herdr(&[
        "agent",
        "start",
        &name,
        "--kind",
        "claude",
        "--pane",
        &pane,
        "--",
        "--session-id",
        &session_id,
        "-n",
        &name,
    ])?;
    let info = json!({
        "name": name,
        "pane": pane,
        "session_id": session_id,
        "branch": if edit { Value::from(branch) } else { Value::Null },
        "started": now(),
    });
    fs::write(folder.join("meta.json"), format!("{info:#}\n")).map_err(|e| e.to_string())?;
    let real = folder.canonicalize().map_err(|e| e.to_string())?;
    send(
        folder,
        &format!(
            "You are a supervised session. Your folder is {}: use it as your task state (your logs and return.md go \
             there, your own children in its sessions/). Read brief.md there and follow it.",
            real.display()
        ),
    )?;
    println!("started {name} in pane {pane} (claude --resume {session_id})");
    Ok(())
}

fn send(folder: &Path, message: &str) -> Result<()> {
    herdr(&["agent", "prompt", &meta_name(folder)?, message])?;
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(folder.join("sent.md"))
        .and_then(|mut log| write!(log, "## {}\n{message}\n\n", now()))
        .map_err(|e| e.to_string())
}

fn returned(folder: &Path) -> String {
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

fn metas(dir: &Path, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            metas(&path, found);
        } else if path.file_name().is_some_and(|n| n == "meta.json") {
            found.push(path);
        }
    }
}

fn status(root: &Path) -> Result<()> {
    let mut found = Vec::new();
    metas(root, &mut found);
    for path in found {
        let folder = path.parent().unwrap_or(root);
        let info = meta(folder)?;
        let name = info["name"].as_str().unwrap_or_default();
        let state = herdr(&["agent", "get", name]).map_or_else(
            |_| "gone".to_string(),
            |r| r["agent"]["agent_status"].as_str().unwrap_or("unknown").to_string(),
        );
        let depth = folder
            .strip_prefix(root)
            .map_or(0, |rel| rel.iter().filter(|p| *p == "sessions").count());
        let resume = if state == "gone" {
            format!(
                "  (claude --resume {})",
                info["session_id"].as_str().unwrap_or_default()
            )
        } else {
            String::new()
        };
        println!(
            "{}{name:24} {state:8} return: {}{resume}",
            "  ".repeat(depth),
            returned(folder)
        );
    }
    Ok(())
}

fn wait(folders: &[PathBuf]) -> Result<()> {
    let mut waits: Vec<(&PathBuf, String, Child)> = Vec::new();
    for folder in folders {
        let name = meta_name(folder)?;
        let child = Command::new("herdr")
            .args(["agent", "wait", &name])
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("herdr: {e}"))?;
        waits.push((folder, name, child));
    }
    loop {
        let settled = waits
            .iter_mut()
            .position(|(_, _, child)| child.try_wait().ok().flatten().is_some());
        if let Some(at) = settled {
            let (folder, name, mut child) = waits.swap_remove(at);
            for (_, _, other) in &mut waits {
                let _ = other.kill();
                let _ = other.wait();
            }
            let mut out = String::new();
            child.stdout.take().map(|mut s| s.read_to_string(&mut out));
            let parsed: Value = serde_json::from_str(&out).unwrap_or(Value::Null);
            let state = parsed["result"]["agent"]["agent_status"].as_str().unwrap_or("unknown");
            println!("{name} is {state}; return: {}", returned(folder));
            return Ok(());
        }
        sleep(Duration::from_millis(200));
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if std::env::var("HERDR_ENV").as_deref() != Ok("1") {
        eprintln!("session: not inside a herdr pane (HERDR_ENV is not 1); supervised sessions need herdr");
        return ExitCode::FAILURE;
    }
    let done = match cli.command {
        Cmd::Start { dir, edit, cwd } => std::env::current_dir()
            .map_err(|e| e.to_string())
            .and_then(|here| start(&dir, edit, &cwd.unwrap_or(here))),
        Cmd::Send { dir, message } => send(&dir, &message),
        Cmd::Status { dir } => status(&dir),
        Cmd::Wait { dirs } => wait(&dirs),
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("session: {e}");
            ExitCode::FAILURE
        }
    }
}
