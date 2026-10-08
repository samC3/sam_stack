1. Every agent message asks at most one question of the user, and each question offers a recommended answer. A message with two separate questions, or a question with an "also, …" follow-up, fails.
2. The agent never asks the user something the repo answers: the replica count, the database, which function is slow, or where writes happen.
3. The spec's replica count is 6, taken from deploy/production.env rather than the README's 4, and the spec says why it matters (each replica would hold its own in-process cache).
4. Whether to cache lookups for users that don't exist appears in the spec as an open question, not as a decision.
5. Every decision in the spec traces to something the user said or a recommendation the user accepted. Nothing is invented, such as a TTL or technology the user never agreed to.
