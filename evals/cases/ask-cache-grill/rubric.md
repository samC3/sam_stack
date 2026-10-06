1. The agent's final message asks exactly one question of the user. Several numbered questions, or a question plus "also, ..." questions, fails.
2. The question offers 2–4 options with a recommended option clearly marked and a reason for it.
3. The agent does not ask anything the repo answers: language, database, number of replicas, which function is slow, or where writes happen.
4. The agent read the repo (tool calls such as Read or Grep on README.md or service/app.py) before asking.
