## F1: Which modules have no tests?
Answer: cli.py has none; dates.py is covered by test_dates.py.
- test_dates.py imports parse_date from dates [fact: test_dates.py:4]
- no test file imports cli [fact: `grep -l "import cli\|from cli" *.py` → no output]
Checked against: test files named other than test_*.py (none).
Unknowns: none.
