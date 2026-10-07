# Findings

## F1: Is there evidence that "review work against its goals in a fresh session" recurs?
Answer: Only the user's request. No transcripts mention it.
- The user says it is used "most commonly for PRs at the moment" [fact: /make-skill args, this conversation]
- No match for "adversarial" or review requests in ~/.claude/projects/*/*.jsonl [fact: `grep -l -i adversarial */*.jsonl` → no output]
- The repo has no PRs to sample [fact: `gh pr list --state all` → empty]
Checked against: transcripts, gh PR history.
Unknowns: how often it runs; the user's word is the evidence.

## F2: Does an existing skill, or a verb with parameters, already cover it?
Answer: `judge` (stance find-defects) covers the judging part, but not finding the goals cold or re-checking the author's claims. That makes it a composition: a workflow.
- judge takes criteria "given, or derived in step 1", and fixes them before looking at the subject [fact: kernel/judge/SKILL.md:12,19]
- judge sends anything runnable to `prove` [fact: kernel/judge/SKILL.md:21]
- In a new session the goals aren't in the conversation; they're in the PR body, linked issue, or work/<slug>/spec.md, decisions.md, tickets [fact: KERNEL.md State section; kernel/distill/SKILL.md formats `spec`, `tickets`, `pr`]. Recovering them is `ground`'s job [inference from those]
- The only existing workflow is make-skill [fact: `ls workflows/`]
Checked against: judge-clean-diff eval (tests judge alone on "Review the last commit") [fact: evals/cases/judge-clean-diff/prompt.md]
Unknowns: none that change the layer.

## F3: Which layer?
Answer: workflow. It wires ground → judge (→ prove) → distill and adds no new kind of uncertainty-removal, so the verb admission rule doesn't apply. [inference from F2 and KERNEL.md Layers]

## F4: Trigger overlap
Answer: "review this" and "tear this apart" already route to judge [fact: kernel/judge/SKILL.md:3]. The new skill needs distinct phrases and a case where it must not fire. [inference]
