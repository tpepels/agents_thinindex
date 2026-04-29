use std::path::PathBuf;

use clap::Parser;

// This Clap parser is the single source of truth for `wi` CLI behavior and help text.
// `src/bin/wi.rs` uses it to parse arguments.

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
  wi refs PromptService
  wi pack PromptService
  wi impact PromptService
  wi bench
  wi .headerNavigation -t css_class
  wi -t css_variable -- --paper-bg
  wi '#mainHeader' -t html_id
  wi 'Tests' -t section
",
    next_line_help = false
)]
pub struct WiArgs {
    #[arg(
        required = true,
        num_args = 1..,
        help = "Search term or subcommand, e.g. HeaderNavigation, refs PromptService, pack PromptService, impact PromptService, bench, --css-variable"
    )]
    pub terms: Vec<String>,

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
        help = "Filter by index source. Values are usually tree_sitter or extras"
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WiCommand {
    Search,
    Refs,
    Pack,
    Impact,
    Bench,
}

impl WiArgs {
    pub fn command(&self) -> WiCommand {
        match self.terms.first().map(String::as_str) {
            Some("refs") if self.terms.len() > 1 => WiCommand::Refs,
            Some("pack") if self.terms.len() > 1 => WiCommand::Pack,
            Some("impact") if self.terms.len() > 1 => WiCommand::Impact,
            Some("bench") if self.terms.len() == 1 => WiCommand::Bench,
            _ => WiCommand::Search,
        }
    }

    pub fn query(&self) -> String {
        match self.command() {
            WiCommand::Search => self.terms.join(" "),
            WiCommand::Refs | WiCommand::Pack | WiCommand::Impact => self.terms[1..].join(" "),
            WiCommand::Bench => String::new(),
        }
    }

    pub fn usage_query(&self) -> String {
        match self.command() {
            WiCommand::Search => self.query(),
            WiCommand::Refs => format!("refs {}", self.query()),
            WiCommand::Pack => format!("pack {}", self.query()),
            WiCommand::Impact => format!("impact {}", self.query()),
            WiCommand::Bench => "bench".to_string(),
        }
    }
}
