check "skill written"                        test -f workflows/investigate/SKILL.md
check "layer is workflow"                    has workflows/investigate/SKILL.md "^Layer:[[:space:]]*workflow"
check "an eval covers it"                    grep -Eq "^covers:.*investigate" evals/tasks/*/task.md
check "lint passes on the result"            "$EVALS_BIN" lint .
check "state says the paid eval wasn't run"  has "$WORK/*/proofs.md" "[Nn]ot run|NOT RUN|not been run|[Uu]nproven"

check "invoked make-skill"                     invoked make-skill
check "invoked ground"                         invoked ground
check "invoked distill"                        invoked distill
check "invoked prove"                          invoked prove
