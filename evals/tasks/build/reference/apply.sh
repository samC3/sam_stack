# A correct build outcome, used by `evals/run.py --graders`.
S="$WORK/slugify"
git checkout -qb slugify
cat >> textkit.py <<'PY'


def slugify(text):
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")
PY
sed -i.bak '1a\
\
import re' textkit.py && rm textkit.py.bak
cat > test_slugify.py <<'PY'
import unittest

from textkit import slugify


class SlugifyTest(unittest.TestCase):
    def test_joins_words_with_single_hyphens(self):
        self.assertEqual(slugify("  Hello, World! "), "hello-world")

    def test_text_without_letters_or_digits_is_empty(self):
        self.assertEqual(slugify("!!!"), "")
PY
git add -A && git commit -qm "01: slugify (P2)"
cat > "$S/proofs.md" <<'MD'
## P1: slugify meets spec criteria 1–4
Observed:
    ImportError: cannot import name 'slugify' from 'textkit'
Verdict: fails

## P2: slugify meets spec criteria 1–4
Observed:
    Ran 4 tests ... OK
Verdict: holds
MD
cat > "$S/verdicts.md" <<'MD'
## J1: ticket 01 diff (find-defects)
Result: no confirmed findings.
MD
cat > "$S/pr.md" <<'MD'
# PR: slugify
What changed: textkit.slugify (ticket 01). Proved by P2. Ticket 02 waits on O1.
MD
