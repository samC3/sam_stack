## D1: Retry failed card charges up to 3 times
Chose: 3 retries. Because: matches the gateway's recommended limit.
Rejected: unlimited retries (risk of repeated customer notifications from the bank).

## D2: Exponential backoff starting at 1 minute
Chose: 1 min, 2 min, 4 min. Because: accepted recommendation.
Depends on: D1

## O1: Should the customer be emailed after the final failed retry?
Default if unanswered: no email; support sees failures in the admin panel.
