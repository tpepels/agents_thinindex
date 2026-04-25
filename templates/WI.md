# WI

Run `build_index` before discovery and after each phase/structural change.
Search first: `wi <term>`; 
filters: `--type`, `--lang`, `--path`, `--limit`.
For terms starting with `--`, use `wi -- --term`.
If no result, rerun `build_index` and retry before scanning.
Read only returned files unless insufficient.