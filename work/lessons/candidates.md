# Lesson candidates

## A request to compare or choose between options is answered inline without invoking judge
Seen: 2026-10-08, eval case adversarial-review-not-for-compare r1 ("Compare approach_a.py and approach_b.py. Which one should I keep…") made no Skill call; the agent ran and timed both files and picked one. Run: ~/.cache/sam_stack-evals/20261008-085510-6096. judge's description already lists "comparing approaches" and "which is better" (kernel/judge/SKILL.md:3). The case's strict check (judge must fire) is kept on purpose. Needs: a second independent incident (e.g. a second failing run of that case)

## The State registry lint only accepts literal file names, so distill's `<format>.md` outputs (review.md, …) are rejected
Seen: 2026-10-08, writing workflows/adversarial-review/SKILL.md with `Output: … work/<slug>/review.md` made `evals/lint.sh` report "Output writes work/<slug>/review.md, which KERNEL.md's State section doesn't register", though KERNEL.md registers `<format>.md` for distill. Worked around by phrasing that Output as "distill's work/<slug>/<format>.md with format review". Needs: nothing more; it is reproduced every time by putting `work/<slug>/review.md` in a skill's Output and running `evals/lint.sh`, so codify can act on it now
