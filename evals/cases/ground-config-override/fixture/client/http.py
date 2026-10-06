from client import settings


def get(url, fetch):
    """Fetch url, retrying up to settings.RETRIES times after the first failure."""
    last_error = None
    for _ in range(settings.RETRIES + 1):
        try:
            return fetch(url, timeout=settings.TIMEOUT_S)
        except IOError as e:
            last_error = e
    raise last_error
