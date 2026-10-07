#!/usr/bin/env bash
# Structural check of every skill against the contract in kernel/KERNEL.md.
# Free and deterministic: run it before any eval that costs tokens.
# Usage: evals/lint.sh [repo-root]   (default: this repo)
set -uo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
errors=0
err() { echo "  $1"; errors=$((errors + 1)); }

# The kernel's State section is the registry of files skills write under work/.
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

  if [ -n "$kernel_state" ]; then
    for p in $(grep -E '^Output:' <<<"$contract" | grep -oE 'work/[^ ;,)]+\.md'); do
      rel="${p#work/<slug>/}"
      grep -qF "\`$rel\`" <<<"$kernel_state" || err "Output writes $p, which KERNEL.md's State section doesn't register"
    done
  fi

  ls -d "$ROOT"/evals/cases/"$dir"-*/ >/dev/null 2>&1 || err "no eval case in evals/cases/$dir-*/"
done

for c in "$ROOT"/evals/cases/*/; do
  [ -d "$c" ] || continue
  [ -f "$c/prompt.md" ] || { echo "${c#$ROOT/}"; err "case missing prompt.md"; }
  [ -f "$c/check.sh" ] || [ -f "$c/rubric.md" ] || { echo "${c#$ROOT/}"; err "case has neither check.sh nor rubric.md"; }
done

if [ "$errors" -eq 0 ]; then echo "lint: ok"; else echo "lint: $errors error(s)"; exit 1; fi
