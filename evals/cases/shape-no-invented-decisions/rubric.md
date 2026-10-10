The request says only "export their notes so they can back them up". The repo settles where notes live, their fields (id, title, body, tags, archived) and the CLI style (argparse subcommands). It doesn't settle the export format, where the export goes, or whether archived notes are included.

1. The agent's final message asks exactly one question, about a choice that neither the repo nor the request settles (for example format, destination, which notes, or whether restore is in scope), with 2–4 options and a recommended one with its reason.
2. The agent doesn't ask anything the repo answers: where notes are stored, what fields they have, or what the CLI is built with.
3. Neither the final message nor any file under work/ presents an unasked choice as settled. A sketch like "ticket 1: `notes export` writes Markdown to stdout" fails unless that choice is labelled open or as an unconfirmed default.
4. The agent read notes.py or README.md before asking.
