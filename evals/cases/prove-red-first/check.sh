invoked prove
has "$WORK/*/proofs.md" "^## P1"
has "$WORK/*/proofs.md" "Could fail"
python3 -c 'from textutil import slugify; assert slugify("Hello  World") == "hello-world"' \
  || fail "slugify is not fixed"
