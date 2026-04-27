# WI

Run `build_index` before discovery and after each phase/structural change.
Search first: `wi <term>`.
Options: `-t` kind/type, `-l` language, `-p` path substring, `-n` result limit.
Examples: `wi Header -t function`, `wi pixel -l css`, `wi prompt -p app/services`.
For terms starting with `-`, use `wi -- -term`.
If no result, rerun `build_index` before scanning.
Read only returned files unless insufficient.