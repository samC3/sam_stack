# skills_v3

A small set of composable agent skills, built up from an atomic core.

```
kernel/
  KERNEL.md          # rules every skill obeys: layers, skill contract, state, evidence, principles
  ground/SKILL.md    # what's true        → work/<slug>/findings.md   (F1…)
  ask/SKILL.md       # what the human wants → work/<slug>/decisions.md (D1…, O1…)
  distill/SKILL.md   # write it down for a named reader
  prove/SKILL.md     # does it work       → work/<slug>/proofs.md     (P1…)
  judge/SKILL.md     # is it good / which one → work/<slug>/verdicts.md (J1…)
```

## Rules

1. **The kernel must be in context wherever these skills are used.** Include `kernel/KERNEL.md` in the `AGENTS.md` that the session reads, either your global one or a repo's own:

   ```markdown
   @~/code/skills_v3/kernel/KERNEL.md
   ```

   `@` imports work in Claude Code. For a tool that doesn't support them, paste the contents of `KERNEL.md` instead and re-paste it when the file changes. If the tool reads `CLAUDE.md` rather than `AGENTS.md`, make `CLAUDE.md` contain just `@AGENTS.md`.

2. **Install skills by linking, not copying,** so edits here take effect everywhere:

   ```sh
   for s in ~/code/skills_v3/kernel/*/; do ln -s "$s" ~/.claude/skills/"$(basename "$s")"; done
   ```

3. **Every skill follows the contract in `KERNEL.md`.** New skills are compositions of the kernel verbs. Add a new verb only when the kernel's admission rule allows it.
