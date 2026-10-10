# notes

A command-line notes app. Python 3, standard library only.

    python3 notes.py add "Title" "Body" --tag work
    python3 notes.py list [--tag work] [--archived]
    python3 notes.py archive <id>

Notes live in a SQLite database at `~/.notes/notes.db`.
