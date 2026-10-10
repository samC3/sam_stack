"""Small text helpers."""


def word_count(text):
    """Return the number of whitespace-separated words in text."""
    # split() with no argument collapses runs of whitespace
    return len(text.split())


def initials(name):
    """Return the uppercase initials of each word in name."""
    # skip empty parts so double spaces don't produce empty initials
    return "".join(part[0].upper() for part in name.split() if part)
