# Decisions

## D1: Export format
Chose: one JSON file, the same shape as notes.json. Because: it restores without a converter.
Rejected: Markdown (lossy for tags and archived).

## D2: Destination
Chose: `notes export <path>` writes to the given path and refuses to overwrite an existing file. Because: accepted recommendation.
Rejected: stdout (too easy to clobber with a redirect).

## D3: Which notes
Chose: all notes, archived included. Because: it's a backup.
Informed by: F1
