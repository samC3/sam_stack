Codify this: skill authors keep leaving contract fields effectively blank. Twice now a SKILL.md went through with `Stop:` followed by nothing but a tab, and `evals/lint.sh` said ok. Make sure skill authors don't do that again.

Repro: `python3 -c "import re,pathlib;p=pathlib.Path('kernel/notes/SKILL.md');p.write_text(re.sub(r'(?m)^Stop:.*$','Stop:\t',p.read_text()))" && bash evals/lint.sh` prints `lint: ok`. Restore the file afterwards with `git checkout kernel/notes/SKILL.md`.
