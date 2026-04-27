# WI

Run `build_index` before discovery and after each phase/structural change. Search first: `wi <term>`.
Options:
* `t <kind>`: filter by kind/type
* `l <lang>`:filter by language
* `p <path>`:filter by path substring
* `n <n>`:result limit
* `v`:verbose output
Examples: `wi Header -t function`, `wi pixel -l css`, `wi prompt -p app/services`.
For terms starting with `-`, use `wi -- -term`.
If no result, rerun `build_index` before scanning.
Read only returned files unless insufficient.