# WI

Repository search rules:
  Run `build_index` before broad discovery and after structural changes.
  Use `wi <term>` before grep/find/ls/Read to locate code.
  `wi` returns repo-local file:line landmarks; Read only returned files unless insufficient.
  If `wi` misses, rerun `build_index` once and retry before falling back.
  For terms starting with `-`, use `wi -- <term>`, e.g. `wi -- --css-variable`.

Examples:
  wi IndexRecord
  wi build_index
  wi .headerNavigation -t css_class
  wi -t css_variable -- --paper-bg
  wi '#mainHeader' -t html_id
  wi 'Tests' -t section


Search the repo-local thin code index and return file/line landmarks

Usage: wi [OPTIONS] <QUERY>

Arguments:
  <QUERY>  Search term, e.g. HeaderNavigation, PromptService, --css-variable

Options:
  -t <KIND>      Filter by indexed record kind. Common kinds: class, function, method, css_class, css_variable, html_id, html_class, html_tag, data_attribute, section, checklist, link, todo, fixme, keyframes
  -l <EXT>       Filter by file extension/language. Use extension-style values: py, rs, js, jsx, ts, tsx, css, html, md
  -p <PATH>      Filter by path substring, e.g. src, tests, frontend/components
  -s <SOURCE>    Filter by index source. Values are usually ctags or extras
  -n <N>         Limit result count, e.g. -n 10
  -v             Show verbose output with kind, language, source, and text
  -r <REPO>      Directory inside the repository [default: .]
  -h, --help     Print help
  -V, --version  Print version
