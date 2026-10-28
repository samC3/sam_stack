use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};
use evals::{comments, eval, lint, transcript};

#[derive(Parser)]
#[command(about = "Lint the skills and eval tasks, and run the evals")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(about = "Free: check every skill, eval task and code file against KERNEL.md and STANDARDS.md")]
    Lint { root: Option<PathBuf> },
    #[command(about = "Run eval trials; they spend real tokens")]
    Run {
        #[command(flatten)]
        tasks: Tasks,
        #[arg(long, default_value_t = 1, help = "trials per task")]
        repeat: usize,
        #[arg(long, default_value_t = 4, help = "trials run in parallel")]
        jobs: usize,
    },
    #[command(about = "Free: each task's checks must fail on its fixture and pass on its reference")]
    Graders {
        #[command(flatten)]
        tasks: Tasks,
    },
    #[command(about = "Exit 0 if the transcript has a Skill call for the skill")]
    Invoked { transcript: PathBuf, skill: String },
    #[command(about = "List comments on lines added since a git revision, in the current repo")]
    Comments { since: String },
}

#[derive(clap::Args)]
struct Tasks {
    #[arg(help = "task names (default: every task)")]
    names: Vec<String>,
    #[arg(long, help = "take KERNEL.md and the skills from another checkout")]
    root: Option<PathBuf>,
    #[arg(long, hide = true)]
    evals: Option<PathBuf>,
}

fn checkout() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

fn out_dir() -> PathBuf {
    let base = std::env::var_os("EVALS_OUT").map_or_else(
        || {
            let cache = std::env::var_os("XDG_CACHE_HOME").map_or_else(
                || PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache"),
                PathBuf::from,
            );
            cache.join("sam_stack-evals")
        },
        PathBuf::from,
    );
    let stamp = Command::new("date")
        .arg("+%Y%m%d-%H%M%S")
        .output()
        .map(|o| o.stdout)
        .unwrap_or_default();
    base.join(format!(
        "{}-{}",
        String::from_utf8_lossy(&stamp).trim(),
        std::process::id()
    ))
}

fn evals(tasks: Tasks, repeat: usize, jobs: usize, graders: bool) -> Result<bool, String> {
    let evals_dir = tasks.evals.unwrap_or_else(|| checkout().join("evals"));
    let skills = tasks
        .root
        .unwrap_or_else(|| evals_dir.parent().unwrap_or(&evals_dir).to_path_buf());
    let ctx = eval::Ctx {
        skills: skills
            .canonicalize()
            .map_err(|e| format!("{}: {e}", skills.display()))?,
        evals: evals_dir,
        out: out_dir(),
        exe: std::env::current_exe().map_err(|e| e.to_string())?,
        env: std::env::vars().collect(),
    };
    let known = ctx.tasks();
    if let Some(unknown) = tasks.names.iter().find(|n| !known.contains(n)) {
        return Err(format!("no task named '{unknown}'"));
    }
    let names = if tasks.names.is_empty() { known } else { tasks.names };
    eval::run_all(&ctx, &names, repeat, jobs, graders)
}

fn exit(passed: Result<bool, String>) -> ExitCode {
    match passed {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("evals: {e}");
            ExitCode::from(2)
        }
    }
}

fn main() -> ExitCode {
    exit(match Cli::parse().command {
        Cmd::Lint { root } => {
            let errors = lint::lint(&root.unwrap_or_else(checkout));
            for e in &errors {
                println!("{e}");
            }
            println!(
                "{}",
                if errors.is_empty() {
                    "lint: ok".into()
                } else {
                    format!("lint: {} error(s)", errors.len())
                }
            );
            Ok(errors.is_empty())
        }
        Cmd::Run { tasks, repeat, jobs } => evals(tasks, repeat, jobs, false),
        Cmd::Graders { tasks } => evals(tasks, 1, 1, true),
        Cmd::Invoked { transcript, skill } => std::fs::read_to_string(&transcript)
            .map(|t| transcript::invoked(&t, &skill))
            .map_err(|e| format!("{}: {e}", transcript.display())),
        Cmd::Comments { since } => std::env::current_dir().map_err(|e| e.to_string()).and_then(|repo| {
            let found = comments::added_since(&repo, &since)?;
            for line in &found {
                println!("{line}");
            }
            Ok(found.is_empty())
        }),
    })
}
