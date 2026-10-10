#!/usr/bin/env python3
"""Start, message, list and wait on supervised Claude Code sessions in herdr.

Each session is a folder, and its children live in its sessions/ subfolder, so
the supervision tree is the folder tree:

  <dir>/brief.md     what the session is asked to do (the supervisor writes it)
  <dir>/meta.json    herdr agent name, pane, Claude session id, branch
  <dir>/sent.md      every message sent to it, in order
  <dir>/return.md    the session's return: status, ids, answer (it writes this)
  <dir>/*.md         the session's own logs: the folder is its task state ($WORK/<slug>)
  <dir>/sessions/    its children

Usage:
  session.py start <dir> [--edit] [--cwd PATH]   needs <dir>/brief.md; --edit gives it a worktree
  session.py send <dir> <message>
  session.py status <dir>                        every session in or under <dir>, as a tree
  session.py wait <dir> [<dir> ...]              until one of them is idle, done or blocked

Run it from a herdr pane (HERDR_ENV=1). The herdr agent name is the folder's path
below the first sessions/, joined with "-": .../sessions/parser/sessions/lexer is
parser-lexer.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import time
import uuid
from datetime import datetime
from pathlib import Path


def die(msg):
    sys.exit(f"session: {msg}")


def herdr(*args):
    """Run one herdr command; returns its JSON result."""
    done = subprocess.run(["herdr", *args], capture_output=True, text=True)
    if done.returncode:
        die(f"herdr {' '.join(args[:2])} failed: {(done.stderr or done.stdout).strip()}")
    return json.loads(done.stdout or "{}").get("result", {})


def agent_name(folder):
    parts = folder.resolve().parts
    if "sessions" not in parts:
        die(f"{folder} is not under a sessions/ folder")
    name = "-".join(p for p in parts[parts.index("sessions") + 1:] if p != "sessions")
    if not re.fullmatch(r"[a-z][a-z0-9_-]{0,31}", name):
        die(f"herdr agent name '{name}' must match [a-z][a-z0-9_-]{{0,31}}; use shorter lowercase folder names")
    return name


def meta(folder):
    path = folder / "meta.json"
    if not path.exists():
        die(f"{folder} has no meta.json; start it first")
    return json.loads(path.read_text())


def start(folder, edit, cwd):
    if not (folder / "brief.md").exists():
        die(f"write {folder}/brief.md first")
    if (folder / "meta.json").exists():
        die(f"{folder} is already started; use send")
    name = agent_name(folder)
    if edit:
        made = herdr("worktree", "create", "--cwd", cwd, "--branch", f"supervise/{name}", "--label", name, "--no-focus")
    else:
        made = herdr("tab", "create", "--workspace", os.environ.get("HERDR_WORKSPACE_ID", ""), "--cwd", cwd,
                     "--label", name, "--no-focus")
    pane = made["root_pane"]["pane_id"]
    session_id = str(uuid.uuid4())
    herdr("agent", "start", name, "--kind", "claude", "--pane", pane, "--", "--session-id", session_id, "-n", name)
    (folder / "meta.json").write_text(json.dumps({
        "name": name, "pane": pane, "session_id": session_id,
        "branch": f"supervise/{name}" if edit else None, "started": datetime.now().isoformat(timespec="seconds"),
    }, indent=2) + "\n")
    send(folder, f"You are a supervised session. Your folder is {folder.resolve()}: use it as your task state "
                 "(your logs and return.md go there, your own children in its sessions/). Read brief.md there and follow it.")
    print(f"started {name} in pane {pane} (claude --resume {session_id})")


def send(folder, message):
    herdr("agent", "prompt", meta(folder)["name"], message)
    with open(folder / "sent.md", "a") as log:
        log.write(f"## {datetime.now().isoformat(timespec='seconds')}\n{message}\n\n")


def returned(folder):
    """The status line of the session's return.md; '-' if there is none, and
    'stale' if it was written before the last message sent to the session."""
    path, sent = folder / "return.md", folder / "sent.md"
    if not path.exists():
        return "-"
    lines = path.read_text().splitlines()
    status = next((line.split(":", 1)[1].strip() for line in lines if line.lower().startswith("status:")), "?")
    return f"stale ({status})" if sent.exists() and path.stat().st_mtime < sent.stat().st_mtime else status


def status(root):
    for path in sorted(root.rglob("meta.json")):
        folder, info = path.parent, json.loads(path.read_text())
        done = subprocess.run(["herdr", "agent", "get", info["name"]], capture_output=True, text=True)
        state = json.loads(done.stdout)["result"]["agent"]["agent_status"] if done.returncode == 0 else "gone"
        depth = folder.relative_to(root).parts.count("sessions")
        resume = f"  (claude --resume {info['session_id']})" if state == "gone" else ""
        print(f"{'  ' * depth}{info['name']:24} {state:8} return: {returned(folder)}{resume}")


def wait(folders):
    waits = {f: subprocess.Popen(["herdr", "agent", "wait", meta(f)["name"]], stdout=subprocess.PIPE, text=True)
             for f in folders}
    while True:
        for folder, proc in waits.items():
            if proc.poll() is not None:
                for other in waits.values():
                    other.kill()
                result = json.loads(proc.stdout.read() or "{}").get("result", {})
                state = result.get("agent", {}).get("agent_status", "unknown")
                print(f"{meta(folder)['name']} is {state}; return: {returned(folder)}")
                return
        time.sleep(1)


def main():
    parser = argparse.ArgumentParser(description="Supervised Claude Code sessions in herdr (see the top of this file).")
    commands = parser.add_subparsers(dest="command", required=True)
    p = commands.add_parser("start")
    p.add_argument("dir", type=Path)
    p.add_argument("--edit", action="store_true", help="give the session its own worktree and branch")
    p.add_argument("--cwd", default=os.getcwd())
    p = commands.add_parser("send")
    p.add_argument("dir", type=Path)
    p.add_argument("message")
    p = commands.add_parser("status")
    p.add_argument("dir", type=Path)
    p = commands.add_parser("wait")
    p.add_argument("dirs", type=Path, nargs="+")
    args = parser.parse_args()

    if os.environ.get("HERDR_ENV") != "1":
        die("not inside a herdr pane (HERDR_ENV is not 1); supervised sessions need herdr")
    if args.command == "start":
        start(args.dir, args.edit, args.cwd)
    elif args.command == "send":
        send(args.dir, args.message)
    elif args.command == "status":
        status(args.dir)
    else:
        wait(args.dirs)


if __name__ == "__main__":
    main()
