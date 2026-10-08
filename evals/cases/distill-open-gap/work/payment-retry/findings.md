## F1: Where are card charges made?
Answer: billing/charge.py `charge()`; a failed charge raises ChargeFailed and nothing retries it.
- charge() raises ChargeFailed on a non-ok gateway result [fact: billing/charge.py:8]
- No caller catches ChargeFailed to retry [fact: `grep -rn ChargeFailed` → only billing/charge.py]
