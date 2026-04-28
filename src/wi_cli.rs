use std::path::PathBuf;

use clap::{CommandFactory, Parser};

// This Clap parser is the single source of truth for `wi` CLI behavior and help text.
// `src/bin/wi.rs` uses it to parse arguments.
// `wi-init` renders repo-local `WI.md` from this same definition via `wi_help_text()`.
// Keep option help explicit: it is written into repo-local `WI.md` for agents.

#[derive(Debug, Parser)]
#[command(
    name = "wi",
    version,
    about = "Search the repo-local thin code index and return file/line landmarks",
    before_help = "\
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
",
    next_line_help = false
)]
pub struct WiArgs {
    #[arg(help = "Search term, e.g. HeaderNavigation, PromptService, --css-variable")]
    pub query: String,

    #[arg(
        short = 't',
        value_name = "KIND",
        help = "Filter by indexed record kind. Common kinds: class, function, method, css_class, css_variable, html_id, html_class, html_tag, data_attribute, section, checklist, link, todo, fixme, keyframes"
    )]
    pub kind: Option<String>,

    #[arg(
        short = 'l',
        value_name = "EXT",
        help = "Filter by file extension/language. Use extension-style values: py, rs, js, jsx, ts, tsx, css, html, md"
    )]
    pub lang: Option<String>,

    #[arg(
        short = 'p',
        value_name = "PATH",
        help = "Filter by path substring, e.g. src, tests, frontend/components"
    )]
    pub path: Option<String>,

    #[arg(
        short = 's',
        value_name = "SOURCE",
        help = "Filter by index source. Values are usually ctags or extras"
    )]
    pub source: Option<String>,

    #[arg(short = 'n', value_name = "N", help = "Limit result count, e.g. -n 10")]
    pub limit: Option<usize>,

    #[arg(
        short = 'v',
        help = "Show verbose output with kind, language, source, and text"
    )]
    pub verbose: bool,

    #[arg(
        short = 'r',
        value_name = "REPO",
        default_value = ".",
        help = "Directory inside the repository"
    )]
    pub repo: PathBuf,
}

pub fn wi_help_text() -> String {
    let mut command = WiArgs::command().term_width(120);
    command.render_help().to_string()
}
