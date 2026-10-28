T="$SAM_STACK_WORK/todo"
mkdir -p "$T/days"
cat > "$T/days/$(date +%F).md" <<MD
# $(date +%F)
TODO     Update the onboarding docs  [repo]
TODO     Write the Q3 roadmap  [repo]
TODO     Clean up old feature flags  [repo]
TODO     Renew the passport  [_global]
TODO     Fix the checkout bug  [repo]
MD
cat > "$T/backlog.md" <<'MD'
TODO     Learn Rust macros  [_global]
MD
