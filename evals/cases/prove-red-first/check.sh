invoked prove
has "work/*/proofs.md" "^## P1"
has "work/*/proofs.md" "Could fail"
python3 -c 'from textutil import slugify; assert slugify("Hello  World") == "hello-world"' \
  || fail "slugify is not fixed"
