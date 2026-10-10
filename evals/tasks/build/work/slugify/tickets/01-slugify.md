# 01: slugify

Goal: `textkit.slugify(text)` turns a title into a slug.
Acceptance: spec criteria 1–4 (D1, D2).
Verify: `python3 -m unittest && python3 -c "from textkit import slugify; assert slugify('  Hello, World! ') == 'hello-world'; assert slugify('!!!') == ''"`
Depends on: nothing
