# A correct shape outcome, used by `evals/run.py --graders`.
P="$WORK/notes-export"; mkdir -p "$P/tickets"
cat > "$P/findings.md" <<'MD'
## F1: Where do notes live, and what does a note contain?
Answer: A JSON list in notes.json beside notes.py; each note has id, title, body, tags, archived.
- STORE = notes.json beside the script [fact: notes.py:5]
- add() writes the five fields [fact: notes.py:18]
MD
cat > "$P/decisions.md" <<'MD'
## D1: Export is JSON in the same shape as notes.json
## D2: `notes export <path>` refuses to overwrite an existing file
## D3: Archived notes are included
## O1: Is an import/restore command part of this work?
Default if unanswered: no; a later ticket.
MD
cat > "$P/verdicts.md" <<'MD'
## J1: spec and tickets (find-defects)
Result: no confirmed defects.
MD
cat > "$P/spec.md" <<'MD'
# Spec: notes export
Outcome: `notes export <path>` writes every note, archived included (D3), as JSON in the notes.json shape (D1, F1), and refuses an existing path (D2).
Open questions: O1, whether restore is part of this work.
MD
cat > "$P/tickets/01-export-command.md" <<'MD'
# 01: export command
Acceptance: D1–D3.
Verify: `python3 notes.py add a b && python3 notes.py export /tmp/x.json && python3 -c "import json; assert len(json.load(open('/tmp/x.json')))==1"`
Depends on: nothing
MD
