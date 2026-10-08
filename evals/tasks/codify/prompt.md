Three lessons from this week. Codify each one.

1. Skill authors keep leaving contract fields effectively blank. Twice now a SKILL.md went through with `Stop:` followed by nothing but a tab, and `evals/lint.py` said ok. Repro: `python3 -c "import re,pathlib;p=pathlib.Path('kernel/notes/SKILL.md');p.write_text(re.sub(r'(?m)^Stop:.*$','Stop:\t',p.read_text()))" && python3 evals/lint.py` prints `lint: ok`. Restore the file afterwards with `git checkout kernel/notes/SKILL.md`.

2. The brief skill invented a decision in yesterday's eval run: it stated a refund policy that nobody has decided. That's exactly what it must never do. The evidence is in eval-runs/2026-10-01/.

3. Three times now, log entries written by different verbs used relative time ("yesterday", "the last run", "earlier today"), and a later session misread them. The incidents are in incidents/.

Don't run any paid evals.
