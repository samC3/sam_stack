mkdir -p evals/tasks .cargo
grep -v '"kernel/supervise",' "$ROOT/Cargo.toml" > Cargo.toml
cp "$ROOT/Cargo.lock" "$ROOT/rustfmt.toml" .
cp "$ROOT/.cargo/config.toml" .cargo/
cp -R "$ROOT/evals/Cargo.toml" "$ROOT/evals/src" "$ROOT/evals/lib.sh" evals/
printf 'target/\n' > .gitignore
cargo metadata -q --format-version 1 --offline > /dev/null
git add -A && git commit -qm "Add eval tooling"
