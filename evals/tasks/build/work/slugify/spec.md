# Spec: slugify

Problem: titles need to become URL-safe slugs.

Outcome and acceptance: `textkit.slugify(text: str) -> str` (D1)
1. Lowercases the text.
2. Each run of characters that aren't ASCII letters or digits becomes one hyphen.
3. No leading or trailing hyphens: `"  Hello, World! "` → `"hello-world"`.
4. Text with no letters or digits gives `""`.

Decisions: D1 (a function in textkit.py), D2 (ASCII only; accented letters count as separators).
Out of scope: transliterating accented letters.
Open questions: O1, how truncation to a maximum length should cut. Default: at a hyphen boundary.
