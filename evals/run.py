#!/usr/bin/env python3
"""Run the eval tasks.

Usage: evals/run.py [--repeat N] [--jobs J] [--root DIR] [task ...]
       evals/run.py --graders [task ...]
  no names    every task in evals/tasks/
  --repeat N  trials per task (default 1); --jobs J parallel trials (default 4)
  --root DIR  take KERNEL.md and the skills from another checkout of this repo
              (e.g. `git worktree add` of HEAD) to compare a baseline
  --graders   free: each task's checks must fail on the untouched fixture and
              pass on its reference solution

A task is evals/tasks/<name>/:
  task.md     frontmatter (covers, turns), then what it's for
  prompt.md   the first user message
  user.md     a simulated user's hidden situation; makes the task multi-turn (optional)
  fixture/    files copied into a fresh git repo (optional)
  work/       task state copied into the run's $WORK, outside the repo (optional)
  setup.sh    run in the repo after the first commit (optional)
  checks.sh   named outcome checks: `check "<label>" <command ...>` (lib.sh helpers)
  rubric.md   numbered dimensions, each graded PASS / FAIL / UNKNOWN by a model
  reference/apply.sh   produces a correct outcome in the repo, for --graders

The run repo gets the kernel the way a real repo does: KERNEL.md imported from
AGENTS.md, and each skill linked into .claude/skills. Only project settings
load, so user-level skills and memory don't interfere.
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from lint import frontmatter

EVALS = Path(__file__).resolve().parent
ROOT = EVALS.parent
TASKS = EVALS / "tasks"
# Outside this repo: a run repo nested in it would inherit its AGENTS.md.
OUT = (Path(os.environ.get("EVALS_OUT") or Path(os.environ.get("XDG_CACHE_HOME") or Path.home() / ".cache") / "sam_stack-evals")
       / f"{time.strftime('%Y%m%d-%H%M%S')}-{os.getpid()}")
ALLOWED_TOOLS = "Bash(git:*) Bash(python3:*) Bash(ls:*) Bash(cat:*) Bash(mkdir:*) Bash(grep:*) Bash(find:*) Bash(chmod:*) Bash(bash:*) Bash(sh:*) Bash(evals/:*)"


def claude(prompt, cwd, env, *flags):
    """One headless Claude Code call. Returns (stdout, stderr)."""
    done = subprocess.run(["claude", "-p", prompt, "--setting-sources", "project", *flags],
                          cwd=cwd, env=env, capture_output=True, text=True)
    return done.stdout, done.stderr


def events(jsonl):
    for line in jsonl.splitlines():
        try:
            yield json.loads(line)
        except ValueError:
            pass


def digest(transcript):
    """The transcript as one line per assistant text, tool call and tool result, for the grader."""
    for event in events(transcript):
        content = (event.get("message") or {}).get("content")
        for part in content if isinstance(content, list) else []:
            kind = part.get("type") if isinstance(part, dict) else None
            if event["type"] == "assistant" and kind == "text":
                yield "ASSISTANT: " + part["text"]
            elif event["type"] == "assistant" and kind == "tool_use":
                yield f"TOOL {part['name']}: {json.dumps(part['input'])[:500]}"
            elif event["type"] == "user" and kind == "tool_result":
                result = part.get("content")
                yield "RESULT: " + (result if isinstance(result, str) else json.dumps(result))[:500]


def forget_sessions_in(path):
    """Claude Code keeps per-project state for every directory a session runs in
    (~/.claude/projects/<path>, /private/tmp/claude-<uid>/<path>, with every
    non-alphanumeric character of the path turned into "-"). Each run makes fresh
    directories, so remove their state; the transcript is saved in the run folder."""
    real = path.resolve()
    assert OUT.resolve() in real.parents, f"refusing to forget sessions outside {OUT}"
    encoded = re.sub(r"[^A-Za-z0-9]", "-", str(real))
    shutil.rmtree(Path.home() / ".claude" / "projects" / encoded, ignore_errors=True)
    shutil.rmtree(Path(f"/private/tmp/claude-{os.getuid()}") / encoded, ignore_errors=True)


class Trial:
    """One run of one task, in its own folder: repo/, work/ (its $SAM_STACK_WORK), and logs."""

    def __init__(self, task, folder, skills_root):
        self.task, self.src, self.dir, self.skills_root = task, TASKS / task, folder, skills_root
        self.repo = folder / "repo"
        self.state = folder / "work"
        # The project name is the repo's directory name, so $WORK is work/repo.
        self.env = {**os.environ, "ROOT": str(ROOT), "SAM_STACK_WORK": str(self.state), "WORK": str(self.state / "repo")}
        self.turns = 0

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.repo, capture_output=True, text=True, check=True).stdout

    def bash(self, script, **env):
        subprocess.run(["bash", str(script)], cwd=self.repo, env={**self.env, **env}, check=True)

    def log(self, name, text):
        with open(self.dir / name, "a") as f:
            f.write(text)

    def prepare(self):
        """Build the run repo and the task state, and commit the starting point."""
        (self.state / "repo").mkdir(parents=True)
        self.repo.mkdir()
        if (self.src / "fixture").is_dir():
            shutil.copytree(self.src / "fixture", self.repo, dirs_exist_ok=True)
        if (self.src / "work").is_dir():
            shutil.copytree(self.src / "work", self.state / "repo", dirs_exist_ok=True)
        shutil.copy(self.skills_root / "kernel" / "KERNEL.md", self.repo / "KERNEL.md")
        (self.repo / "AGENTS.md").write_text("@KERNEL.md\n")
        installed = self.repo / ".claude" / "skills"
        installed.mkdir(parents=True, exist_ok=True)
        for skill in sorted(self.skills_root.glob("kernel/*/SKILL.md")) + sorted(self.skills_root.glob("workflows/*/SKILL.md")):
            (installed / skill.parent.name).symlink_to(skill.parent)
        self.git("init", "-q")
        self.git("add", "-A")
        self.git("commit", "-qm", "fixture")
        if (self.src / "setup.sh").exists():
            self.bash(self.src / "setup.sh")
        self.base = self.git("rev-parse", "HEAD").strip()

    def checks(self, name, transcript=os.devnull):
        """Run checks.sh; returns its PASS/FAIL lines and saves them to <name>.txt."""
        lines = []
        if (self.src / "checks.sh").exists():
            done = subprocess.run(["bash", "-c", 'source "$1"; source "$2"', "_", EVALS / "lib.sh", self.src / "checks.sh"],
                                  cwd=self.repo, env={**self.env, "TRANSCRIPT": str(transcript)}, capture_output=True, text=True)
            lines = [line for line in (done.stdout + done.stderr).splitlines() if line.startswith(("PASS ", "FAIL "))]
        if (self.repo / "work").exists():
            lines.append("FAIL task state stays out of the repo: repo has a work/ folder")
        self.log(f"{name}.txt", "".join(line + "\n" for line in lines))
        return lines

    def agent_turn(self, message, session=None):
        """One headless agent turn. Returns the session id to resume."""
        self.turns += 1
        out, err = claude(message, self.repo, self.env, *(["--resume", session] if session else []),
                          "--output-format", "stream-json", "--verbose", "--permission-mode", "acceptEdits",
                          "--add-dir", str(self.skills_root / "kernel"), "--add-dir", str(self.skills_root / "workflows"),
                          "--add-dir", str(self.state), "--allowedTools", ALLOWED_TOOLS, "--disallowedTools", "AskUserQuestion")
        self.log(f"turn-{self.turns}.jsonl", out)
        self.log("transcript.jsonl", out)
        self.log("stderr.log", err)
        # Everything the agent said this turn, not only its last text block:
        # options and the question often arrive in separate blocks.
        said = [part["text"] for e in events(out) if e.get("type") == "assistant" and not e.get("parent_tool_use_id")
                for part in e["message"].get("content", []) if part.get("type") == "text"]
        self.log("conversation.md", "AGENT:\n" + "\n".join(said) + "\n\n")
        return next((e.get("session_id") for e in reversed(list(events(out))) if e.get("type") == "result"), None)

    def user_reply(self):
        """The simulated user's next message, or <<END>>."""
        prompt = (EVALS / "prompts" / "user.md").read_text().format(
            situation=(self.src / "user.md").read_text(), conversation=(self.dir / "conversation.md").read_text())
        (self.dir / "user").mkdir(exist_ok=True)
        reply, err = claude(prompt, self.dir / "user", self.env)
        self.log("stderr.log", err)
        self.log("conversation.md", f"USER:\n{reply}\n\n")
        return reply

    def outcome(self):
        """What the run left behind, for the grader: the task state and the repo's changes."""
        parts = [f"=== state: {f.relative_to(self.state)}\n{f.read_text(errors='replace')}"
                 for f in sorted(self.state.rglob("*")) if f.is_file()]
        parts.append("=== repo: git diff since setup\n" + self.git("diff", self.base))
        for f in self.git("ls-files", "--others", "--exclude-standard").splitlines():
            if not f.startswith(".claude/"):
                head = (self.repo / f).read_text(errors="replace").splitlines()[:200]
                parts.append(f"=== repo: {f} (new)\n" + "\n".join(head))
        return "\n".join(parts)

    def grade(self, transcript_digest):
        """One model grader; returns one 'n: VERDICT — evidence' line per rubric dimension."""
        outcome = self.outcome()
        self.log("outcome.txt", outcome)
        prompt = (EVALS / "prompts" / "grader.md").read_text().format(
            rubric=(self.src / "rubric.md").read_text(), transcript=transcript_digest, outcome=outcome)
        (self.dir / "grader").mkdir()
        out, err = claude(prompt, self.dir / "grader", self.env)
        lines = re.findall(r"^\d+: (?:PASS|FAIL|UNKNOWN)\b.*", out, re.M) or ["0: FAIL — grader output unreadable"]
        self.log("grade.md", out + err)
        return lines


def run_trial(task, rep, skills_root):
    """Run one trial; returns (passed, report)."""
    t = Trial(task, OUT / task / f"r{rep}", skills_root)
    t.prepare()
    session = t.agent_turn((t.src / "prompt.md").read_text())
    if (t.src / "user.md").exists():
        max_turns = int((frontmatter((t.src / "task.md").read_text()) or {}).get("turns", 8))
        while t.turns < max_turns and session:
            reply = t.user_reply()
            if "<<END>>" in (line.strip() for line in reply.splitlines()):
                break
            session = t.agent_turn(reply, session)
    transcript = (t.dir / "transcript.jsonl").read_text()
    transcript_digest = "\n".join(digest(transcript))
    t.log("digest.txt", transcript_digest)

    checks = t.checks("checks", t.dir / "transcript.jsonl")
    rubric = t.grade(transcript_digest) if (t.src / "rubric.md").exists() else []
    for folder in (t.repo, t.dir / "grader", t.dir / "user"):
        forget_sessions_in(folder)

    check_fails = [c for c in checks if c.startswith("FAIL")]
    rubric_passes = [r for r in rubric if ": PASS" in r]
    unknown = sum(": UNKNOWN" in r for r in rubric)
    passed = not check_fails and len(rubric_passes) == len(rubric)
    report = (f"{task:26} r{rep:<3} {'PASS' if passed else 'FAIL'}  checks {len(checks) - len(check_fails)}/{len(checks)}"
              f"  rubric {len(rubric_passes)}/{len(rubric)}{f' ({unknown} unknown)' if unknown else ''}  turns {t.turns}")
    details = [f"    check  {c}" for c in check_fails] + [f"    rubric {r}" for r in rubric if r not in rubric_passes]
    return passed, "\n".join([report, *details])


def test_graders(task, skills_root):
    """Free: the checks must fail on the untouched fixture and all pass on the reference."""
    t = Trial(task, OUT / "graders" / task, skills_root)
    reference = t.src / "reference" / "apply.sh"
    if not reference.exists():
        return True, f"{task:26} SKIP  no reference/apply.sh"
    t.prepare()
    untouched = t.checks("untouched")
    t.bash(reference)
    after = t.checks("reference")
    fails_before = sum(c.startswith("FAIL") for c in untouched)
    fails_after = [c for c in after if c.startswith("FAIL")]
    passed = fails_before > 0 and not fails_after
    report = (f"{task:26} {'PASS' if passed else 'FAIL'}  untouched fails {fails_before}/{len(untouched)}"
              f"  reference passes {len(after) - len(fails_after)}/{len(after)}")
    return passed, "\n".join([report, *(f"    reference {c}" for c in fails_after)])


def main():
    parser = argparse.ArgumentParser(description="Run the eval tasks (see the top of this file).")
    parser.add_argument("tasks", nargs="*", help="task names (default: all)")
    parser.add_argument("--repeat", type=int, default=1)
    parser.add_argument("--jobs", type=int, default=4)
    parser.add_argument("--root", type=Path, default=ROOT, help="checkout to take the kernel and skills from")
    parser.add_argument("--graders", action="store_true")
    args = parser.parse_args()
    tasks = args.tasks or sorted(p.name for p in TASKS.iterdir() if p.is_dir())
    for task in tasks:
        if not (TASKS / task).is_dir():
            parser.error(f"no task named '{task}'")
    OUT.mkdir(parents=True)

    skills_root = args.root.resolve()
    results = []  # (task, passed)
    with open(OUT / "results.txt", "a") as log:
        with ThreadPoolExecutor(1 if args.graders else args.jobs) as pool:
            if args.graders:
                trials = {pool.submit(test_graders, task, skills_root): task for task in tasks}
            else:
                trials = {pool.submit(run_trial, task, rep, skills_root): task
                          for task in tasks for rep in range(1, args.repeat + 1)}
            for done in as_completed(trials):
                task = trials[done]
                try:
                    passed, text = done.result()
                except Exception as e:  # a broken task fails its trial, not the whole run
                    passed, text = False, f"{task:26} ERROR  {e!r}"
                print(text, flush=True)
                log.write(text + "\n")
                results.append((task, passed))
        if not args.graders:
            print("\nSummary (passing trials / trials):")
            for task in tasks:
                print(f"  {task:26} {sum(p for t, p in results if t == task)}/{args.repeat}")
    print(f"Runs: {OUT}")
    sys.exit(0 if all(p for _, p in results) else 1)


if __name__ == "__main__":
    main()
