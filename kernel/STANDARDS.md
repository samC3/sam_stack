# Standards

The bar for code that any skill in this set writes: in this repo (its tools, the deterministic scripts for repeatable tasks, the eval runner and the eval checks), and, through `build`, in any repo. Each rule names what enforces it, because a rule nothing checks is a wish (KERNEL.md, principle 6).

## Every repo

1. **Tests first, through the public interface.** Every behaviour has a test that was seen failing before the code that passes it existed (`prove`).
2. **Coverage: at least 95% of lines,** measured by the repo's coverage tool.
3. **Mutation: no missed mutants.** Every mutant the repo's mutation tool makes is caught by a test. A survivor means a test is missing or too weak: add or sharpen the test. Exclude a mutant only when no test could tell it apart from the original (an equivalent mutant), and give the reason where the exclusion is listed.
4. **Zero comments.** No comments and no docstrings, in code or tests. Say it with a name, a type, a smaller function or a test instead. The only exceptions are directives a tool reads: a shebang, `# shellcheck`, `noqa`, `type:`, an encoding line.
5. **No errors or warnings** from the repo's compiler, linter, formatter or type checker.

Use the repo's own tools for each rule. Where a repo has no tool for one, say so in the findings and the PR, and don't add one unasked.

## This repo

- **Rust for anything with logic:** skill tools, deterministic scripts for repeatable tasks, and the eval runner. A tool lives with its owner, as a crate in the root `Cargo.toml` workspace: the evals' in `evals/`, a skill's in its own folder (`kernel/supervise/`).
- **Shell only for glue:** a task's `checks.sh`, `setup.sh` and `reference/apply.sh`, a task's `bin/` fakes, and `gate.sh`. Glue is a sequence of commands. Once a script needs logic worth testing, it becomes Rust, with tests.
- **The compiler is the first reviewer.** `[workspace.lints]` forbids `unsafe`, makes every warning an error, and denies clippy's `all` and `pedantic` groups.
- **Errors are values.** Return `Result` with a message that says what failed and on what path. Report it once, in `main`, and exit non-zero. Don't panic on bad input.
- **Tests:** unit tests in `<module>/tests.rs` beside the module (`#[cfg(test)] mod tests;`), and tests of a binary through its CLI in the crate's `tests/`. Fake an external program (`claude`, `herdr`) with a script first on the `PATH`, the way the eval tasks do.
- **Few dependencies.** Prefer the standard library. A crate must earn its place: today that is `clap` and `serde_json`.

## The gate

`./gate.sh` runs every check, cheapest first, and stops at the first failure. Run it before committing code.

| Step | Enforces |
|---|---|
| `cargo fmt --check` | formatting |
| `cargo clippy --all-targets` | 5 |
| `cargo test` | 1 |
| `cargo lint` | 4 (comments in `.rs`, `.sh`, `.py` and `bin/` files, outside eval fixtures), plus every skill and eval task |
| `cargo llvm-cov --fail-under-lines 95` | 2 |
| `cargo mutants` | 3 |

A mutant that makes the tests hang (cargo-mutants exit code 3, a timeout) counts as caught. The mutation step is the slow one. While iterating, run it on what you changed: `cargo mutants --file <path>`.
