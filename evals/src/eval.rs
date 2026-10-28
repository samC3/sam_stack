use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::Write as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, mpsc};
use std::thread;

use crate::lint::frontmatter;
use crate::transcript;

const ALLOWED_TOOLS: &str = "Bash(git:*) Bash(python3:*) Bash(cargo:*) Bash(ls:*) Bash(cat:*) Bash(mkdir:*) \
                             Bash(grep:*) Bash(find:*) Bash(chmod:*) Bash(bash:*) Bash(sh:*)";

pub struct Ctx {
    pub evals: PathBuf,
    pub skills: PathBuf,
    pub out: PathBuf,
    pub exe: PathBuf,
    pub env: BTreeMap<String, String>,
}

impl Ctx {
    fn root(&self) -> &Path {
        self.evals.parent().unwrap_or(&self.evals)
    }

    pub fn tasks(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.evals.join("tasks"))
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

pub type Outcome = Result<(bool, String), String>;

fn run(cmd: &mut Command) -> Result<std::process::Output, String> {
    let program = cmd.get_program().to_string_lossy().into_owned();
    cmd.output().map_err(|e| format!("{program}: {e}"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn readable(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_default();
    if bytes.contains(&0) {
        format!("(binary, {} bytes)", bytes.len())
    } else {
        text(&bytes)
    }
}

fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(at) = rest.find('{') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        if let Some((key, value)) = values.iter().find(|(key, _)| rest.starts_with(&format!("{{{key}}}"))) {
            out.push_str(value);
            rest = &rest[key.len() + 2..];
        } else {
            out.push('{');
            rest = &rest[1..];
        }
    }
    out + rest
}

fn is_verdict(line: &str) -> bool {
    let Some((n, rest)) = line.split_once(": ") else {
        return false;
    };
    !n.is_empty()
        && n.chars().all(|c| c.is_ascii_digit())
        && ["PASS", "FAIL", "UNKNOWN"].iter().any(|v| {
            rest.strip_prefix(v)
                .is_some_and(|after| after.chars().next().is_none_or(|c| !(c.is_alphanumeric() || c == '_')))
        })
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let entries = fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?;
    for entry in entries.flatten() {
        let (src, dest) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            copy_tree(&src, &dest)?;
        } else {
            fs::copy(&src, &dest).map_err(|e| format!("{}: {e}", src.display()))?;
        }
    }
    Ok(())
}

fn files_under(dir: &Path, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files_under(&path, found);
        } else {
            found.push(path);
        }
    }
}

fn skill_dirs(skills: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for layer in ["kernel", "workflows"] {
        let mut dirs: Vec<PathBuf> = fs::read_dir(skills.join(layer))
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.join("SKILL.md").is_file())
            .collect();
        dirs.sort();
        found.extend(dirs);
    }
    found
}

pub struct Trial<'a> {
    ctx: &'a Ctx,
    src: PathBuf,
    dir: PathBuf,
    repo: PathBuf,
    state: PathBuf,
    env: BTreeMap<String, String>,
    max_turns: usize,
    turns: usize,
    base: String,
}

impl<'a> Trial<'a> {
    pub fn new(ctx: &'a Ctx, task: &str, dir: PathBuf) -> Trial<'a> {
        let src = ctx.evals.join("tasks").join(task);
        let front = frontmatter(&fs::read_to_string(src.join("task.md")).unwrap_or_default()).unwrap_or_default();
        let mut env: BTreeMap<String, String> = ctx
            .env
            .iter()
            .filter(|(k, _)| !k.starts_with("HERDR_"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if src.join("bin").is_dir() {
            let path = env.get("PATH").cloned().unwrap_or_default();
            env.insert("PATH".into(), format!("{}:{path}", src.join("bin").display()));
        }
        for pair in front
            .get("env")
            .map(String::as_str)
            .unwrap_or_default()
            .split_whitespace()
        {
            if let Some((key, value)) = pair.split_once('=') {
                env.insert(key.into(), value.into());
            }
        }
        let state = dir.join("work");
        env.insert("ROOT".into(), ctx.root().display().to_string());
        env.insert("SAM_STACK_WORK".into(), state.display().to_string());
        env.insert("WORK".into(), state.join("repo").display().to_string());
        env.insert("EVALS_BIN".into(), ctx.exe.display().to_string());
        Trial {
            ctx,
            repo: dir.join("repo"),
            max_turns: front.get("turns").and_then(|t| t.parse().ok()).unwrap_or(8),
            src,
            dir,
            state,
            env,
            turns: 0,
            base: String::new(),
        }
    }

    fn command(&self, program: &str) -> Command {
        let mut cmd = Command::new(program);
        cmd.env_clear().envs(&self.env).current_dir(&self.repo);
        cmd
    }

    fn git(&self, args: &[&str]) -> Result<String, String> {
        let out = run(self.command("git").args(args))?;
        if !out.status.success() {
            return Err(format!("git {}: {}", args.join(" "), text(&out.stderr).trim()));
        }
        Ok(text(&out.stdout))
    }

    fn bash(&self, script: &Path) -> Result<(), String> {
        let out = run(self.command("bash").arg(script))?;
        if !out.status.success() {
            return Err(format!("{} failed: {}", script.display(), text(&out.stderr).trim()));
        }
        Ok(())
    }

    fn log(&self, name: &str, body: &str) -> Result<(), String> {
        let path = self.dir.join(name);
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut f| f.write_all(body.as_bytes()))
            .map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn prepare(&mut self) -> Result<(), String> {
        let io = |e: std::io::Error| e.to_string();
        fs::create_dir_all(self.state.join("repo")).map_err(io)?;
        fs::create_dir_all(&self.repo).map_err(io)?;
        if self.src.join("fixture").is_dir() {
            copy_tree(&self.src.join("fixture"), &self.repo)?;
        }
        if self.src.join("work").is_dir() {
            copy_tree(&self.src.join("work"), &self.state.join("repo"))?;
        }
        fs::copy(self.ctx.skills.join("kernel/KERNEL.md"), self.repo.join("KERNEL.md")).map_err(io)?;
        fs::write(self.repo.join("AGENTS.md"), "@KERNEL.md\n").map_err(io)?;
        let installed = self.repo.join(".claude/skills");
        fs::create_dir_all(&installed).map_err(io)?;
        for skill in skill_dirs(&self.ctx.skills) {
            std::os::unix::fs::symlink(&skill, installed.join(skill.file_name().unwrap_or_default())).map_err(io)?;
        }
        self.git(&["init", "-q"])?;
        self.git(&["add", "-A"])?;
        self.git(&["commit", "-qm", "fixture"])?;
        if self.src.join("setup.sh").is_file() {
            self.bash(&self.src.join("setup.sh"))?;
        }
        self.base = self.git(&["rev-parse", "HEAD"])?.trim().to_string();
        Ok(())
    }

    pub fn checks(&self, name: &str, transcript: &Path) -> Result<Vec<String>, String> {
        let mut lines = Vec::new();
        let script = self.src.join("checks.sh");
        if script.is_file() {
            let out = run(self
                .command("bash")
                .args(["-c", "source \"$1\"; source \"$2\"", "_"])
                .arg(self.ctx.evals.join("lib.sh"))
                .arg(&script)
                .env("TRANSCRIPT", transcript))?;
            lines = (text(&out.stdout) + &text(&out.stderr))
                .lines()
                .filter(|l| l.starts_with("PASS ") || l.starts_with("FAIL "))
                .map(String::from)
                .collect();
        }
        if self.repo.join("work").exists() {
            lines.push("FAIL task state stays out of the repo: repo has a work/ folder".into());
        }
        self.log(
            &format!("{name}.txt"),
            &lines.iter().fold(String::new(), |all, l| all + l + "\n"),
        )?;
        Ok(lines)
    }

    fn claude(&self, prompt: &str, cwd: &Path, flags: &[&str]) -> Result<(String, String), String> {
        let out = run(self
            .command("claude")
            .args(["-p", prompt, "--setting-sources", "project"])
            .args(flags)
            .current_dir(cwd))?;
        Ok((text(&out.stdout), text(&out.stderr)))
    }

    pub fn agent_turn(&mut self, message: &str, session: Option<&str>) -> Result<Option<String>, String> {
        self.turns += 1;
        let kernel = self.ctx.skills.join("kernel").display().to_string();
        let workflows = self.ctx.skills.join("workflows").display().to_string();
        let state = self.state.display().to_string();
        let mut flags = Vec::new();
        if let Some(session) = session {
            flags.extend(["--resume", session]);
        }
        flags.extend([
            "--output-format",
            "stream-json",
            "--verbose",
            "--permission-mode",
            "acceptEdits",
        ]);
        flags.extend(["--add-dir", &kernel, "--add-dir", &workflows, "--add-dir", &state]);
        flags.extend(["--allowedTools", ALLOWED_TOOLS, "--disallowedTools", "AskUserQuestion"]);
        let (out, err) = self.claude(message, &self.repo, &flags)?;
        self.log(&format!("turn-{}.jsonl", self.turns), &out)?;
        self.log("transcript.jsonl", &out)?;
        self.log("stderr.log", &err)?;
        self.log(
            "conversation.md",
            &format!("AGENT:\n{}\n\n", transcript::said(&out).join("\n")),
        )?;
        Ok(transcript::session_id(&out))
    }

    pub fn user_reply(&self) -> Result<String, String> {
        let prompt = fill(
            &fs::read_to_string(self.ctx.evals.join("prompts/user.md")).map_err(|e| e.to_string())?,
            &[
                (
                    "situation",
                    &fs::read_to_string(self.src.join("user.md")).unwrap_or_default(),
                ),
                (
                    "conversation",
                    &fs::read_to_string(self.dir.join("conversation.md")).unwrap_or_default(),
                ),
            ],
        );
        let cwd = self.dir.join("user");
        fs::create_dir_all(&cwd).map_err(|e| e.to_string())?;
        let (reply, err) = self.claude(&prompt, &cwd, &[])?;
        self.log("stderr.log", &err)?;
        self.log("conversation.md", &format!("USER:\n{reply}\n\n"))?;
        Ok(reply)
    }

    pub fn outcome(&self) -> Result<String, String> {
        let mut files = Vec::new();
        files_under(&self.state, &mut files);
        let mut parts: Vec<String> = files
            .iter()
            .map(|f| {
                let rel = f.strip_prefix(&self.state).unwrap_or(f).display();
                format!("=== state: {rel}\n{}", readable(f))
            })
            .collect();
        parts.push(format!(
            "=== repo: git diff since setup\n{}",
            self.git(&["diff", &self.base])?
        ));
        for file in self.git(&["ls-files", "--others", "--exclude-standard"])?.lines() {
            if !file.starts_with(".claude/") {
                let body = readable(&self.repo.join(file));
                let head: Vec<&str> = body.lines().take(200).collect();
                parts.push(format!("=== repo: {file} (new)\n{}", head.join("\n")));
            }
        }
        Ok(parts.join("\n"))
    }

    pub fn grade(&self, digest: &str) -> Result<Vec<String>, String> {
        let outcome = self.outcome()?;
        self.log("outcome.txt", &outcome)?;
        let prompt = fill(
            &fs::read_to_string(self.ctx.evals.join("prompts/grader.md")).map_err(|e| e.to_string())?,
            &[
                (
                    "rubric",
                    &fs::read_to_string(self.src.join("rubric.md")).unwrap_or_default(),
                ),
                ("transcript", digest),
                ("outcome", &outcome),
            ],
        );
        let cwd = self.dir.join("grader");
        fs::create_dir_all(&cwd).map_err(|e| e.to_string())?;
        let (out, err) = self.claude(&prompt, &cwd, &[])?;
        self.log("grade.md", &(out.clone() + &err))?;
        let lines: Vec<String> = out.lines().filter(|l| is_verdict(l)).map(String::from).collect();
        Ok(if lines.is_empty() {
            vec!["0: FAIL — grader output unreadable".into()]
        } else {
            lines
        })
    }

    fn forget_sessions_in(&self, path: &Path) -> Result<(), String> {
        let real = path.canonicalize().map_err(|e| format!("{}: {e}", path.display()))?;
        let out = self.ctx.out.canonicalize().map_err(|e| e.to_string())?;
        if !real.starts_with(&out) || real == out {
            return Err(format!("refusing to forget sessions outside {}", out.display()));
        }
        let encoded: String = real
            .display()
            .to_string()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        let home = PathBuf::from(self.env.get("HOME").cloned().unwrap_or_default());
        let uid = fs::metadata(&self.dir).map_err(|e| e.to_string())?.uid();
        let _ = fs::remove_dir_all(home.join(".claude/projects").join(&encoded));
        let _ = fs::remove_dir_all(PathBuf::from(format!("/private/tmp/claude-{uid}")).join(&encoded));
        Ok(())
    }
}

pub fn run_trial(ctx: &Ctx, task: &str, rep: usize) -> Outcome {
    let mut t = Trial::new(ctx, task, ctx.out.join(task).join(format!("r{rep}")));
    t.prepare()?;
    let prompt = fs::read_to_string(t.src.join("prompt.md")).map_err(|e| e.to_string())?;
    let mut session = t.agent_turn(&prompt, None)?;
    if t.src.join("user.md").is_file() {
        while t.turns < t.max_turns {
            let Some(id) = session else { break };
            let reply = t.user_reply()?;
            if reply.lines().any(|l| l.trim() == "<<END>>") {
                break;
            }
            session = t.agent_turn(&reply, Some(&id))?;
        }
    }
    let transcript_path = t.dir.join("transcript.jsonl");
    let digest = transcript::digest(&fs::read_to_string(&transcript_path).unwrap_or_default()).join("\n");
    t.log("digest.txt", &digest)?;

    let checks = t.checks("checks", &transcript_path)?;
    let rubric = if t.src.join("rubric.md").is_file() {
        t.grade(&digest)?
    } else {
        Vec::new()
    };
    for folder in [t.repo.clone(), t.dir.join("grader"), t.dir.join("user")] {
        if folder.exists() {
            t.forget_sessions_in(&folder)?;
        }
    }

    let check_fails: Vec<&String> = checks.iter().filter(|c| c.starts_with("FAIL")).collect();
    let rubric_fails: Vec<&String> = rubric.iter().filter(|r| !r.contains(": PASS")).collect();
    let unknown = rubric.iter().filter(|r| r.contains(": UNKNOWN")).count();
    let passed = check_fails.is_empty() && rubric_fails.is_empty();
    let mut report = format!(
        "{task:26} r{rep:<3} {}  checks {}/{}  rubric {}/{}{}  turns {}",
        if passed { "PASS" } else { "FAIL" },
        checks.len() - check_fails.len(),
        checks.len(),
        rubric.len() - rubric_fails.len(),
        rubric.len(),
        if unknown > 0 {
            format!(" ({unknown} unknown)")
        } else {
            String::new()
        },
        t.turns,
    );
    for c in check_fails {
        report = report + "\n    check  " + c;
    }
    for r in rubric_fails {
        report = report + "\n    rubric " + r;
    }
    Ok((passed, report))
}

pub fn test_graders(ctx: &Ctx, task: &str) -> Outcome {
    let mut t = Trial::new(ctx, task, ctx.out.join("graders").join(task));
    let reference = t.src.join("reference/apply.sh");
    if !reference.is_file() {
        return Ok((true, format!("{task:26} SKIP  no reference/apply.sh")));
    }
    t.prepare()?;
    let untouched = t.checks("untouched", Path::new("/dev/null"))?;
    t.bash(&reference)?;
    let after = t.checks("reference", &t.src.join("reference/transcript.jsonl"))?;
    let fails_before = untouched.iter().filter(|c| c.starts_with("FAIL")).count();
    let fails_after: Vec<&String> = after.iter().filter(|c| c.starts_with("FAIL")).collect();
    let passed = fails_before > 0 && fails_after.is_empty();
    let mut report = format!(
        "{task:26} {}  untouched fails {fails_before}/{}  reference passes {}/{}",
        if passed { "PASS" } else { "FAIL" },
        untouched.len(),
        after.len() - fails_after.len(),
        after.len(),
    );
    for c in fails_after {
        report = report + "\n    reference " + c;
    }
    Ok((passed, report))
}

pub fn run_all(ctx: &Ctx, tasks: &[String], repeat: usize, jobs: usize, graders: bool) -> Result<bool, String> {
    fs::create_dir_all(&ctx.out).map_err(|e| format!("{}: {e}", ctx.out.display()))?;
    let queue: Mutex<VecDeque<(String, usize)>> = Mutex::new(if graders {
        tasks.iter().map(|t| (t.clone(), 0)).collect()
    } else {
        tasks
            .iter()
            .flat_map(|t| (1..=repeat).map(move |r| (t.clone(), r)))
            .collect()
    });
    let (send, results) = mpsc::channel();
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ctx.out.join("results.txt"))
        .map_err(|e| e.to_string())?;
    let mut outcomes: Vec<(String, bool)> = Vec::new();
    thread::scope(|scope| {
        for _ in 0..(if graders { 1 } else { jobs.max(1) }) {
            let (queue, send) = (&queue, send.clone());
            scope.spawn(move || {
                while let Some((task, rep)) = queue.lock().ok().and_then(|mut q| q.pop_front()) {
                    let outcome = if graders {
                        test_graders(ctx, &task)
                    } else {
                        run_trial(ctx, &task, rep)
                    };
                    if send.send((task, outcome)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(send);
        for (task, outcome) in results {
            let (passed, report) = outcome.unwrap_or_else(|e| (false, format!("{task:26} ERROR  {e}")));
            println!("{report}");
            let _ = writeln!(log, "{report}");
            outcomes.push((task, passed));
        }
    });
    if !graders {
        let mut summary = String::from("\nSummary (passing trials / trials):\n");
        for task in tasks {
            let n = outcomes.iter().filter(|(t, p)| t == task && *p).count();
            summary = summary + &format!("  {task:26} {n}/{repeat}") + "\n";
        }
        print!("{summary}");
        let _ = write!(log, "{summary}");
    }
    println!("Runs: {}", ctx.out.display());
    Ok(outcomes.iter().all(|(_, p)| *p))
}

#[cfg(test)]
mod tests;
