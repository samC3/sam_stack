# Spec: CSV export

Problem: reports need to be downloadable as CSV.

Outcome and acceptance — `export.to_csv(rows: list[dict]) -> str`:
1. The first line is a header made of the first row's keys, in order.
2. Each row becomes one line, with values in header order.
3. A value containing a comma or a double quote is quoted as RFC 4180 requires: `a,b` → `"a,b"`, `say "hi"` → `"say ""hi"""`.

Out of scope: streaming, other delimiters.
