#!/usr/bin/env python3
"""Tests for session.py against a fake herdr. Free and deterministic.

The fake records every call in herdr.log and answers from environment variables:
FAKE_STATUS (agent get), FAKE_GONE (names herdr no longer knows), FAKE_DONE (the
name whose wait returns; every other wait hangs).
"""
import os
import subprocess
import sys
import tempfile
from pathlib import Path

SESSION = Path(__file__).resolve().parent / "session.py"

FAKE_HERDR = r'''#!/usr/bin/env python3
import json, os, sys, time
args = sys.argv[1:]
with open(os.environ["FAKE_LOG"], "a") as log:
    log.write(" ".join(args) + "\n")
if args[:2] in (["tab", "create"], ["worktree", "create"]):
    print(json.dumps({"result": {"root_pane": {"pane_id": "w1:p9"}}}))
elif args[:2] == ["agent", "get"]:
    if args[2] in os.environ.get("FAKE_GONE", "").split(","):
        sys.exit("agent_not_found")
    print(json.dumps({"result": {"agent": {"agent_status": os.environ.get("FAKE_STATUS", "working")}}}))
elif args[:2] == ["agent", "wait"]:
    if args[2] != os.environ.get("FAKE_DONE"):
        time.sleep(30)
    print(json.dumps({"result": {"agent": {"agent_status": "done"}}}))
else:
    print(json.dumps({"result": {}}))
'''


def run(tmp, *args, **env):
    full = {**os.environ, "PATH": f"{tmp}/bin:{os.environ['PATH']}", "FAKE_LOG": f"{tmp}/herdr.log", "HERDR_ENV": "1", **env}
    done = subprocess.run([sys.executable, str(SESSION), *args], cwd=tmp, env=full, capture_output=True, text=True, timeout=20)
    return done.returncode, done.stdout + done.stderr


def brief(tmp, path):
    folder = Path(tmp) / path
    folder.mkdir(parents=True)
    (folder / "brief.md").write_text("Do the thing.\n")
    return str(folder)


def calls(tmp):
    return (Path(tmp) / "herdr.log").read_text()


def case_start_tab(tmp):
    code, out = run(tmp, "start", brief(tmp, "w/sessions/parser"))
    meta = (Path(tmp) / "w/sessions/parser/meta.json").read_text()
    sent = (Path(tmp) / "w/sessions/parser/sent.md").read_text()
    assert code == 0, out
    assert "tab create" in calls(tmp) and "--label parser" in calls(tmp)
    assert "agent start parser --kind claude --pane w1:p9 -- --session-id" in calls(tmp)
    assert '"branch": null' in meta and "session_id" in meta
    assert "agent prompt parser You are a supervised session" in calls(tmp)
    assert "use it as your task state" in sent and "Read brief.md there" in sent


def case_start_edit_gets_worktree(tmp):
    code, out = run(tmp, "start", brief(tmp, "w/sessions/parser"), "--edit")
    assert code == 0, out
    assert "worktree create" in calls(tmp) and "--branch supervise/parser" in calls(tmp)
    assert "tab create" not in calls(tmp)


def case_start_needs_brief(tmp):
    (Path(tmp) / "w/sessions/parser").mkdir(parents=True)
    code, out = run(tmp, "start", f"{tmp}/w/sessions/parser")
    assert code != 0 and "brief.md first" in out, out


def case_start_twice(tmp):
    folder = brief(tmp, "w/sessions/parser")
    run(tmp, "start", folder)
    code, out = run(tmp, "start", folder)
    assert code != 0 and "already started" in out, out


def case_nested_name(tmp):
    code, out = run(tmp, "start", brief(tmp, "w/sessions/parser/sessions/lexer"))
    assert code == 0 and "agent start parser-lexer" in calls(tmp), out


def case_bad_name(tmp):
    code, out = run(tmp, "start", brief(tmp, "w/sessions/Parser"))
    assert code != 0 and "must match" in out, out


def case_send(tmp):
    folder = brief(tmp, "w/sessions/parser")
    run(tmp, "start", folder)
    code, out = run(tmp, "send", folder, "Test 3 still fails: see P2.")
    assert code == 0, out
    assert "agent prompt parser Test 3 still fails: see P2." in calls(tmp)
    assert "Test 3 still fails" in (Path(folder) / "sent.md").read_text()


def case_status_tree(tmp):
    parser, lexer = brief(tmp, "w/sessions/parser"), brief(tmp, "w/sessions/parser/sessions/lexer")
    run(tmp, "start", parser)
    run(tmp, "start", lexer)
    (Path(parser) / "return.md").write_text("status: done\nids: P1\n")
    code, out = run(tmp, "status", f"{tmp}/w", FAKE_STATUS="idle", FAKE_GONE="parser-lexer")
    lines = out.splitlines()
    assert code == 0, out
    assert lines[0].startswith("  parser ") and "idle" in lines[0] and "return: done" in lines[0], out
    assert lines[1].startswith("    parser-lexer ") and "gone" in lines[1] and "claude --resume" in lines[1], out


def case_return_older_than_last_message_is_stale(tmp):
    folder = brief(tmp, "w/sessions/parser")
    run(tmp, "start", folder)
    (Path(folder) / "return.md").write_text("status: done\n")
    os.utime(Path(folder) / "return.md", (1, 1))
    code, out = run(tmp, "status", folder, FAKE_STATUS="idle")
    assert "return: stale (done)" in out, out
    (Path(folder) / "return.md").write_text("status: done\n")
    code, out = run(tmp, "status", folder, FAKE_STATUS="idle")
    assert "return: done" in out and "stale" not in out, out


def case_wait_returns_first(tmp):
    a, b = brief(tmp, "w/sessions/a"), brief(tmp, "w/sessions/b")
    run(tmp, "start", a)
    run(tmp, "start", b)
    code, out = run(tmp, "wait", a, b, FAKE_DONE="b")
    assert code == 0 and out.startswith("b is done"), out


def case_outside_herdr(tmp):
    code, out = run(tmp, "status", tmp, HERDR_ENV="")
    assert code != 0 and "not inside a herdr pane" in out, out


failed = 0
for name, case in [(n, f) for n, f in list(globals().items()) if n.startswith("case_")]:
    with tempfile.TemporaryDirectory() as tmp:
        (Path(tmp) / "bin").mkdir()
        (Path(tmp) / "bin" / "herdr").write_text(FAKE_HERDR)
        (Path(tmp) / "bin" / "herdr").chmod(0o755)
        (Path(tmp) / "herdr.log").touch()
        try:
            case(tmp)
            print(f"PASS  {name[5:]}")
        except AssertionError as e:
            failed += 1
            print(f"FAIL  {name[5:]}\n        {e}")
print(f"session_test: {failed} failed" if failed else "session_test: ok")
sys.exit(1 if failed else 0)
