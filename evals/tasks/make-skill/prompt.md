Make a workflow skill called `investigate`. It answers a question about the codebase: run `ground` on the question, then `distill` a `summary` for the user. Its key habit is that it never edits code. It should trigger on "investigate", "dig into" and "look into why", and must not trigger on requests to change code.

Those are all the decisions, so don't ask me anything else. Don't run `cargo evals run` because it costs tokens. Running `cargo lint` is fine.
