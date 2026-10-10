import argparse
import json
from pathlib import Path

STORE = Path(__file__).with_name("notes.json")


def load():
    return json.loads(STORE.read_text()) if STORE.exists() else []


def save(notes):
    STORE.write_text(json.dumps(notes, indent=2))


def add(title, body, tags):
    notes = load()
    note = {"id": len(notes) + 1, "title": title, "body": body, "tags": tags, "archived": False}
    notes.append(note)
    save(notes)
    return note


def listing(tag=None, archived=False):
    return [n for n in load() if n["archived"] == archived and (tag is None or tag in n["tags"])]


def archive(note_id):
    notes = load()
    for n in notes:
        if n["id"] == note_id:
            n["archived"] = True
    save(notes)


def main():
    p = argparse.ArgumentParser(prog="notes")
    sub = p.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("add")
    a.add_argument("title")
    a.add_argument("body")
    a.add_argument("--tag", action="append", default=[])
    l = sub.add_parser("list")
    l.add_argument("--tag")
    l.add_argument("--archived", action="store_true")
    r = sub.add_parser("archive")
    r.add_argument("id", type=int)
    args = p.parse_args()
    if args.cmd == "add":
        print(add(args.title, args.body, args.tag)["id"])
    elif args.cmd == "list":
        for n in listing(args.tag, args.archived):
            print(f'{n["id"]}  {n["title"]}  {",".join(n["tags"])}')
    elif args.cmd == "archive":
        archive(args.id)


if __name__ == "__main__":
    main()
