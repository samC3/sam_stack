#!/usr/bin/env bash
# Structural check of every skill against the contract in kernel/KERNEL.md.
# Free and deterministic: run it before any eval that costs tokens.
# Usage: evals/lint.sh [repo-root]   (default: this repo)
set -uo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
errors=0
err() { echo "  $1"; errors=$((errors + 1)); }

# The kernel's State section is the registry of files skills write under $WORK/ and $SAM_STACK_WORK/.
kernel_state="$(awk '/^## State/{s=1; next} s && /^## /{exit} s' "$ROOT/kernel/KERNEL.md" 2>/dev/null)"

for f in "$ROOT"/kernel/*/SKILL.md "$ROOT"/workflows/*/SKILL.md; do
  [ -f "$f" ] || continue
  dir="$(basename "$(dirname "$f")")"
  area="$(basename "$(dirname "$(dirname "$f")")")"
  echo "${f#$ROOT/}"

  [ "$(head -1 "$f")" = "---" ] || err "frontmatter must start on line 1"
  front="$(awk 'NR==1{next} /^---$/{exit} {print}' "$f")"
  name="$(sed -n 's/^name: *//p' <<<"$front")"
  [ "$name" = "$dir" ] || err "frontmatter name '$name' must match directory '$dir'"
  grep -Eq '^description: *.{40,}' <<<"$front" || err "description missing or too short to route on"

  grep -q '^## Contract' "$f" || err "missing '## Contract'"
  contract="$(awk '/^## Contract/{c=1; next} c && /^```/{n++; if (n==2) exit; next} c && n==1' "$f")"
  for field in Layer Input Output Stop Returns; do
    grep -Eq "^$field:[[:space:]]*[^[:space:]]" <<<"$contract" || err "contract missing '$field:'"
  done

  layer="$(sed -n 's/^Layer:[[:space:]]*\([a-z]*\).*/\1/p' <<<"$contract")"
  case "$area:$layer" in
    kernel:verb|kernel:combinator|workflows:workflow) ;;
    *) err "layer '$layer' doesn't belong in $area/" ;;
  esac

  # Task state lives under $WORK/ (KERNEL.md State), never in the project repo.
  repo_work="$(grep -nE '(^|[^[:alnum:]_/.$-])work/' "$f" | head -1)"
  [ -z "$repo_work" ] || err "line ${repo_work%%:*} names a repo-relative work/ path; task state lives under \$WORK/ (KERNEL.md State)"

  if [ -n "$kernel_state" ]; then
    for p in $(grep -E '^Output:' <<<"$contract" | grep -oE '\$(WORK|SAM_STACK_WORK)/[^ ;,)]+\.md'); do
      rel="${p#\$WORK/<slug>/}"
      grep -qF "\`$rel\`" <<<"$kernel_state" || err "Output writes $p, which KERNEL.md's State section doesn't register"
    done
  fi

  covers_re="^Covers:(.*[ ,])?$dir([ ,]|\$)"
  grep -Eqs "$covers_re" "$ROOT"/evals/tasks/*/task.md || err "no eval task covers $dir (a Covers: line in evals/tasks/*/task.md)"
done

# Skills a task may cover: this repo's own, plus any installed in .claude/skills/.
skills=" $(ls -d "$ROOT"/kernel/*/ "$ROOT"/workflows/*/ "$ROOT"/.claude/skills/*/ 2>/dev/null | xargs -n1 basename | sort -u | tr '\n' ' ')"
for t in "$ROOT"/evals/tasks/*/; do
  [ -d "$t" ] || continue
  echo "${t#$ROOT/}"
  [ -f "$t/prompt.md" ] || err "task missing prompt.md"
  [ -f "$t/checks.sh" ] || [ -f "$t/rubric.md" ] || err "task has neither checks.sh nor rubric.md"
  [ ! -f "$t/checks.sh" ] || [ -f "$t/reference/apply.sh" ] || err "task has checks.sh but no reference/apply.sh to test them against"
  grep -Eq '^Kind: *(capability|regression)$' "$t/task.md" 2>/dev/null || err "task.md needs 'Kind: capability' or 'Kind: regression'"
  covers="$(sed -n 's/^Covers: *//p' "$t/task.md" 2>/dev/null | tr ',' ' ')"
  [ -n "$covers" ] || err "task.md needs a 'Covers:' line naming the skills it exercises"
  for c in $covers; do
    case "$skills" in *" $c "*) ;; *) err "Covers names '$c', which isn't a skill in kernel/, workflows/ or .claude/skills/" ;; esac
  done
done

if [ "$errors" -eq 0 ]; then echo "lint: ok"; else echo "lint: $errors error(s)"; exit 1; fi
