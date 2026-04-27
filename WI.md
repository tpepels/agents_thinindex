#WI.md

Agent usage:
  Run `build_index` before discovery and after each phase/structural change.
  Use `wi <term>` before reading files; results are indexed repo landmarks with file/line locations.
  If no result, rerun `build_index` once and retry before scanning.
  Read only returned files unless insufficient.
  For search terms starting with `-`, use `wi -- <term>`, e.g. `wi -- --css-variable`

Search the repo-local thin code index and return file/line landmarks

Usage: wi [OPTIONS] <QUERY>

Arguments:
  <QUERY>
          Search term, e.g. HeaderNavigation, PromptService, --css-variable

Options:
  -t <KIND>
          Filter by indexed record kind. Common kinds: class, function, method, css_class, css_variable, html_id, html_class, html_tag, data_attribute, heading, checklist, link, todo, fixme, keyframes

  -l <EXT>
          Filter by file extension/language. Use extension-style values: py, rs, js, jsx, ts, tsx, css, html, md

  -p <PATH>
          Filter by path substring, e.g. src, tests, frontend/components

  -s <SOURCE>
          Filter by index source. Values are usually ctags or extras

  -n <N>
          Limit result count, e.g. -n 10

  -v
          Show verbose output with kind, language, source, and text

  -r <REPO>
          Directory inside the repository
          
          [default: .]

  -h, --help
          Print help

  -V, --version
          Print version
