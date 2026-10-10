# Findings

## F1: Where do notes live, and what does a note contain?
Answer: In notes.json next to notes.py, as a list of {id, title, body, tags, archived}.
- `STORE = Path(__file__).with_name("notes.json")` [fact: notes.py:5]
- add() writes those five fields [fact: notes.py:18]
