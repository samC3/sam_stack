# Decisions

## D1: slugify is a function in textkit.py
Chose: textkit.slugify(text). Because: textkit already holds the text helpers.

## D2: ASCII only; accented letters count as separators
Chose: ASCII only. Because: no transliteration dependency for now.

## O1: When truncating to a maximum length, cut at a hyphen boundary or hard-cut at the limit?
Default if unanswered: at a hyphen boundary.
