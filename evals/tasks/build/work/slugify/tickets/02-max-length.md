# 02: maximum length

Goal: `slugify(text, max_length)` truncates the slug to at most max_length characters.
Acceptance: the result is never longer than max_length and never ends in a hyphen; how it cuts follows O1.
Verify: `python3 -c "from textkit import slugify; assert len(slugify('a b c d e f', max_length=5)) <= 5"`
Depends on: O1
