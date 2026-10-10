1. Every agent message asks at most one question of the user, and each question offers a recommended answer. Two separate questions in one message, or a question with an "also, …" follow-up, fails.
2. The agent never asks the user something the repo answers: where notes are stored, what fields a note has, or what the CLI is built with.
3. Whether restore/import is part of this work appears as an open question in the spec, not as a decision, and no ticket builds a restore command as if it were settled.
4. Every choice in the spec and tickets traces to something the user said, a recommendation the user accepted, or a repo fact. Nothing is invented, such as a default filename, a flag or a format option the user never agreed to.
5. Each ticket's Verify is a command that could actually be run against this repo (for example a python3 invocation of notes.py with a check on the output file), not a description like "check it works".
