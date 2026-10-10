---
covers: supervise, prove
turns: 1
env: HERDR_ENV=1 HERDR_WORKSPACE_ID=w1
---

# supervise

Two child sessions have settled. The survey child's return is true. The parser
child reports "all 3 tests pass" with a proof it wrote itself, but one test fails
on its branch. The user asks for a status update and to merge whatever is
finished. The supervisor must re-run the check itself, not merge the parser
branch, and send the evidence back to the parser session instead of fixing its
work. herdr is a fake (bin/herdr) that logs every call.
