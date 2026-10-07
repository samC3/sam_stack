# Decisions: make-skill-codify

Settled in the request: name `codify`; layer verb (F1); core rule is principle 6's order; callers are a future retro workflow and make-skill; input is a lesson plus evidence ids; output is the edit plus a record of why. Budget: one eval run for the new case.

## D1: How much evidence is enough to act
Chose: two or more independent incidents, or one incident that a deterministic check reproduces (reproducing it counts as the repeat). A single incident that can't be reproduced is logged as a candidate and nothing changes. Because: accepted recommendation.
Rejected: always require a repeat (would have stalled the lint fix); one incident is enough (would have acted on 1-in-4 noise); a rate threshold (only works for eval failures).
Informed by: F3, F6

## D2: Which edits need the user's sign-off first
Chose: any KERNEL.md edit, and any edit that removes or loosens a rule or check (skill wording, lint rule, check.sh, rubric). codify proposes these and waits. Adding a lint rule, a case or skill wording goes ahead and is recorded. Because: accepted recommendation.
Rejected: sign-off on everything (a retro workflow couldn't run unattended); kernel only (lets a loosening like F5 through silently); no sign-off at all.
Informed by: F5, F7

## D3: How codify picks the enforcement level, and when a note is acceptable
Chose: walk the ladder top-down: lint.sh → case check.sh → runner flags → rubric → SKILL.md text → KERNEL.md text → note. Take the first level that can block or deterministically reject the failure. When only text can change the behaviour, it goes in the skill that owns the behaviour, and in KERNEL.md only if it applies to every skill. The record names each stronger level and why it didn't fit. A note is allowed only when no skill owns the behaviour and nothing can observe it. Because: accepted recommendation.
Rejected: free judgment (drifts toward wording); always pair a mechanism with text (adds text where a lint rule alone works).
Informed by: F7

## D4: Must every codify leave a regression check
Chose: yes. Each codify leaves one check that fails on the original incident, the cheapest kind that can: a bad fixture lint must reject for a lint fix, and a new or existing eval case for a behaviour fix. The original incident counts as the red run. A paid run after the fix follows make-skill's budget: one run, then ask. Because: accepted recommendation.
Rejected: always a new eval case (pays tokens for what a free script checks); behaviour lessons only (leaves lesson B unguarded, F3); optional.
Depends on: D3 · Informed by: F2, F3

## D5: May codify remove or simplify rules
Chose: yes, and it checks for this before adding. If an existing rule already covers the lesson, the fix is a stronger level for that rule, not new text. If the new mechanism makes existing text redundant, codify proposes deleting it. Every removal goes through sign-off (D2). Because: accepted recommendation.
Rejected: remove only when asked; add only.
Depends on: D2

## D6: How codify avoids overfitting to one bad run
Chose: the wording names the class of failure, not the incident's details. The check that accepts the fix must differ on the surface from the incident (a different domain or prompt). Before applying, codify asks one question of the change: would the rule make sense to someone who never saw the incident? Because: accepted recommendation.
Rejected: a held-out eval set (strongest, but new infrastructure; revisit later); rely on D1's threshold alone.
Depends on: D1, D4 · Informed by: F2 (the ask fix's example came straight from the failing run)

## D7: Where the record of why and the candidates live
Chose: a new log, work/<slug>/lessons.md, with ids L<n>. Each entry gives the lesson, its evidence ids, the level chosen, the stronger levels rejected and why, and the regression check. The commit message cites the L id. Candidates waiting for a repeat go to one long-lived work/lessons/candidates.md that is exempt from deletion. Both need a row in the kernel's State table, which make-skill can't add, so this is open (O1) until codify's first run, with user sign-off (D2). Because: accepted recommendation.
Rejected: commit message only (an unreproduced first incident is forgotten); a durable LESSONS.md (a note file that grows forever).
Depends on: D1, D2 · Informed by: F8

## D8: Trigger phrases
Chose: fire on "codify this", "make sure this never happens again", "turn this lesson into a rule", "this keeps happening", "add a lint rule for". Must not fire on "make a skill for X" (make-skill), "remember this" (memory), "write this down" (distill), or a plain "fix this bug" with no lesson attached. Because: accepted recommendation.
Rejected: callers only (users who say "never again" can't reach it); broad correction phrases (compete with memory, and fire on one-offs that D1 rejects).
Depends on: D1

## D9: Habit and eval cases
Chose: three cases, each run once. (1) Strongest level wins: the failure is wording where a mechanism fits; the case is lesson B against the pre-fix lint. (2) Evidence threshold: the failure is acting on one unreproducible incident; the case is a single 1-in-4 failure. (3) Kernel sign-off: the failure is editing KERNEL.md without asking; the case is a cross-skill lesson. The skill's headline habit is (1). Because: the user widened the budget: "if all 3 add value do all three".
Supersedes: the request's budget of one eval run.
Depends on: D1, D2, D3

## O1: Register codify's logs in the kernel
Default if unanswered: codify's first run proposes, for sign-off, a row in KERNEL.md's State table for `lessons.md` (written by codify, ids `L1`, …) and an exemption from the deletion rule for `work/lessons/candidates.md`. Until then the skill defines both itself. make-skill may not edit the kernel.
