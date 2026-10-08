You own the profile service.
- Why: get_profile is slow (around 450 ms). You want repeat reads under 50 ms.
- No new infrastructure: no Redis, no memcached. In-process caching is fine.
- Follower counts can be up to 60 seconds stale.
- A name change must be visible everywhere within a few seconds.
- Whether to cache lookups for users that don't exist: you haven't decided and don't want to now. If asked, say to leave it open.
- You don't remember how many replicas run in production, or other details that are in the repo. If asked, tell the agent to check the repo.
- You're happy to accept the agent's recommendation on anything else.
