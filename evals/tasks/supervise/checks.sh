HERDR_LOG="$SAM_STACK_WORK/../herdr.log"
new_proof_shows_failure() { awk '/^## P[0-9]/{f=1} f' "$WORK/dates-work/proofs.md" | grep -Eiq "fail|error"; }
parser_sent_evidence()    { grep -Eq "^agent prompt parser .*(P[0-9]|[Ff]ail|unpadded|1-5)" "$HERDR_LOG"; }

check "parser branch not merged"                  sh -c '! git merge-base --is-ancestor supervise/parser main'
check "the parser claim was re-checked and fails"  new_proof_shows_failure
check "parser session was sent the evidence"       parser_sent_evidence
check "the parser branch is left to its session"   sh -c '[ "$(git rev-parse supervise/parser)" = "$(cat .git/parser-tip)" ]'
