# Decisions

## D1: Name, layer, flow
Chose: `shape`, a workflow: ground → ask → distill → judge. Because: user specified it in the request.
Informed by: F2

## D2: Output
Chose: always `work/<slug>/spec.md` and `work/<slug>/tickets/NN-<name>.md`, even when there's only one ticket. Because: accepted recommendation. Build gets one input shape, and adversarial-review already reads both (F4).
Rejected: tickets only (no home for the why and out of scope); spec only (build would have to slice the work); choosing per input (build would have to handle two shapes).

## D3: What happens on judge defects
Chose: send each confirmed defect back to the step that owns it (a missing decision to ask, a writing gap to distill), then re-judge. At most 2 rounds, then stop and report what's still open. Because: accepted recommendation.
Rejected: report only (artifacts ship with known holes); loop until clean (no budget).
Depends on: D2

## D4: The habit
Chose: no invented decisions. Every choice in the spec and tickets traces to the input, an F id or a D id. Typical failure: going straight from the idea to tickets and filling unstated choices with assumptions. Because: accepted recommendation.
Rejected: coverage of the input (judge checks it anyway); ground before asking (the kernel and ask's own eval already cover it).

## D5: Triggers
Chose: fires on "shape this", "spec this out", "plan this feature", "turn this idea/doc into tickets", "get this ready to build". Doesn't fire on "write the spec" when the decisions are already logged (distill), or on "grill me" / "help me decide" (ask). The negative case uses settled decisions plus "Write the spec for X". Because: accepted recommendation.
Rejected: also taking "write the spec" (it would re-ask settled questions); explicit name only.
Informed by: F3

## D6: Slug
Chose: default the slug to the feature's kebab-case name, not `shape-<x>`. Because: inferred from F4 (build and review look under the same work/<slug>/); not asked.

## O1: Open questions left after ask
Default if unanswered: carry them into spec.md's Open questions with ask's default. A ticket whose outcome depends on one lists it under Depends on (`O2`).
