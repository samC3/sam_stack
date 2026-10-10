---
name: brief
description: Write a one-page brief for a stakeholder from the decisions and findings logged so far. Use for "write a brief", "brief the team".
---

# Brief

## Contract

```contract
Layer:     verb
Input:     reader; slug
Output:    $WORK/<slug>/brief.md
Stop:      every section cites a decision or finding id, or is marked open
Returns:   the path
```

## Rules

- Never invent a decision. Mark gaps as open questions.
