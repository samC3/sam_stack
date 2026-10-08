check "skill written"                        test -f workflows/investigate/SKILL.md
check "layer is workflow"                    has workflows/investigate/SKILL.md "^Layer:[[:space:]]*workflow"
check "an eval covers it"                    grep -Eq "^Covers:.*investigate" evals/tasks/*/task.md
check "lint passes on the result"            bash evals/lint.sh .
check "state says the paid eval wasn't run"  has "$WORK/*/proofs.md" "[Nn]ot run|NOT RUN|not been run|[Uu]nproven"
