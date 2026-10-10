# Findings

## F1: Is there evidence that "turn an idea or doc into build-ready artifacts" recurs?
Answer: Mainly the user's word. One weak sign that ticketed vertical slices are already in use.
- The user asks for it as a planning step ahead of "an upcoming build workflow" [fact: /make-skill args, this conversation]
- A herdr worktree is named `vertical-slice-01-m0-cli-validated-call` [fact: `ls ~/.herdr/worktrees/that_way`]
- No transcript matches for "write the spec", "ticket this", "vertical slice" or "prd" outside this session [fact: `grep -l -i -E ... ~/.claude/projects/*/*.jsonl` → no output]
Checked against: transcripts, other repos' work/ dirs (none exist).
Unknowns: how often it runs; the user's word is the evidence.

## F2: Does an existing skill, or a verb with parameters, already cover it?
Answer: No single skill does. Each step is an existing verb; the wiring from raw input to build-ready artifacts is new. That makes it a workflow; no new verb is needed.
- `distill` writes `spec` and `tickets` but "never decides anything" and assumes the state is already settled [fact: kernel/distill/SKILL.md:3,21-22,47]
- `ask` resolves intent and reads findings.md first, but writes only decisions.md [fact: kernel/ask/SKILL.md:16,22-23]
- `judge` find-defects reviews an artifact against fixed criteria [fact: kernel/judge/SKILL.md:16,19,25]
- distill's cold-read test checks the reader can act; it doesn't check the artifacts cover the source input [fact: kernel/distill/SKILL.md:39] [inference: judge's added value here is coverage and traceability against the input, not readability]
- Workflows that exist: make-skill, adversarial-review [fact: `ls workflows/`]
Checked against: the kernel's verb admission rule (KERNEL.md Layers); nothing here is a new kind of uncertainty-removal.

## F3: Trigger overlap
Answer: distill already owns "write the spec" and "ticket this up"; ask owns "flesh this out" and "help me decide" [fact: kernel/distill/SKILL.md:3; kernel/ask/SKILL.md:3]. shape needs phrases about starting from raw input ("shape this", "spec out", "plan this out", "turn this doc into tickets") and a case where it must not fire. [inference]

## F4: Who reads shape's output?
Answer: A build workflow that doesn't exist yet, and adversarial-review, which already reads the same files as goals.
- No build workflow exists [fact: `ls workflows/`]
- adversarial-review takes goals from `work/<their-slug>/spec.md`, `decisions.md`, `tickets/` [fact: workflows/adversarial-review/SKILL.md step 2]
- distill's `tickets` format gives each slice Goal · Acceptance · Verify (a command `prove` can run) · Depends on [fact: kernel/distill/SKILL.md:21]
- [inference from the above] shape's slug should be the feature's slug, not `shape-<x>`, so build and review find the files under the same `work/<slug>/`.
