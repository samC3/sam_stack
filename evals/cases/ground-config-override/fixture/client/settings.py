import os

from client.defaults import DEFAULT_RETRIES, DEFAULT_TIMEOUT_S

RETRIES = int(os.environ.get("CLIENT_RETRIES", DEFAULT_RETRIES))
TIMEOUT_S = int(os.environ.get("CLIENT_TIMEOUT_S", DEFAULT_TIMEOUT_S))
