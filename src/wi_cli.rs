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
#WI.md

Agent usage:
  Run `build_index` before discovery and after each phase/structural change.
  Use `wi <term>` before reading files; results are indexed repo landmarks with file/line locations.
  If no result, rerun `build_index` once and retry before scanning.
  Read only returned files unless insufficient.
  For search terms starting with `-`, use `wi -- <term>`, e.g. `wi -- --css-variable`",
    next_line_help = false
)]
pub struct WiArgs {
    #[arg(help = "Search term, e.g. HeaderNavigation, PromptService, --css-variable")]
    pub query: String,

    #[arg(
        short = 't',
        value_name = "KIND",
        help = "Filter by indexed record kind. Common kinds: class, function, method, css_class, css_variable, html_id, html_class, html_tag, data_attribute, heading, checklist, link, todo, fixme, keyframes"
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
    WiArgs::command().render_long_help().to_string()
}
