use std::env;

use anyhow::{Context, Result, bail};
use clap::Parser;
use thinindex::{
    bench::{BenchmarkRunOptions, render_benchmark_report, run_benchmark},
    context::{render_impact_command, render_pack_command, render_refs_command},
    doctor::{render_doctor_report, run_doctor},
    indexer::{build_index, find_repo_root, index_is_fresh},
    search::{SearchOptions, format_result, search},
    stats::{self, UsageEvent},
    wi_cli::{WiArgs, WiCommand},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = WiArgs::parse();
    let command = args.command();
    let query = args.query();
    let usage_query = args.usage_query();
    let start = if args.repo.is_absolute() {
        args.repo.clone()
    } else {
        env::current_dir()?.join(&args.repo)
    };
    let root = find_repo_root(&start)?;

    if command == WiCommand::Doctor {
        print!("{}", render_doctor_report(&run_doctor(&root)));
        return Ok(());
    }

    ensure_index_ready_once(&root)?;

    let used_type = args.kind.is_some();
    let used_lang = args.lang.is_some();
    let used_path = args.path.is_some();
    let used_limit = args.limit.is_some();
    let limit = match command {
        WiCommand::Search => args.limit.unwrap_or(30),
        WiCommand::Refs => args.limit.unwrap_or(20),
        WiCommand::Pack => args.limit.unwrap_or(10),
        WiCommand::Impact => args.limit.unwrap_or(15),
        WiCommand::Doctor => args.limit.unwrap_or(0),
        WiCommand::Bench => args.limit.unwrap_or(0),
    };

    let options = SearchOptions {
        kind: args.kind,
        lang: args.lang,
        path: args.path,
        source: args.source,
        limit,
        verbose: args.verbose,
    };

    let mut log_usage = true;
    let result_count = match command {
        WiCommand::Search => {
            let results = search(&root, &query, &options)?;
            let result_count = results.len();

            for result in &results {
                println!("{}", format_result(result, options.verbose));
            }

            result_count
        }
        WiCommand::Refs => {
            let output = render_refs_command(&root, &query, &options)?;
            if !output.text.is_empty() {
                print!("{}", output.text);
            }
            output.result_count
        }
        WiCommand::Pack => {
            let output = render_pack_command(&root, &query, &options)?;
            if !output.text.is_empty() {
                print!("{}", output.text);
            }
            output.result_count
        }
        WiCommand::Impact => {
            let output = render_impact_command(&root, &query, &options)?;
            if !output.text.is_empty() {
                print!("{}", output.text);
            }
            output.result_count
        }
        WiCommand::Doctor => unreachable!("doctor returns before index freshness checks"),
        WiCommand::Bench => {
            let report = run_benchmark(
                &root,
                BenchmarkRunOptions {
                    queries: None,
                    build_duration: None,
                },
            )?;
            print!("{}", render_benchmark_report(&report));
            log_usage = false;
            report.query_count
        }
    };

    if !log_usage {
        return Ok(());
    }

    let event = UsageEvent {
        ts: stats::current_unix_seconds(),
        command: command.usage_category().to_string(),
        query: usage_query.clone(),
        query_len: usage_query.chars().count(),
        result_count,
        hit: result_count > 0,
        used_type,
        used_lang,
        used_path,
        used_limit,
        repo: root.display().to_string(),
        indexed_files: stats::manifest_indexed_files(&root),
    };

    if let Err(error) = stats::append_usage_event(&root, &event) {
        eprintln!("warning: failed to log wi usage: {error:#}");
    }

    Ok(())
}

fn ensure_index_ready_once(root: &std::path::Path) -> Result<()> {
    let rebuild_reason = match index_is_fresh(root) {
        Ok(true) => return Ok(()),
        Ok(false) => "index is stale; repository files changed since the last build".to_string(),
        Err(error) => format!("{error:#}"),
    };

    eprintln!("wi: {rebuild_reason}");
    eprintln!("wi: running `build_index` once, then continuing the command");

    let stats = build_index(root).with_context(|| {
        format!(
            "failed to auto-build index for {}\nwhy: {rebuild_reason}\nnext: fix the indexing error or run `build_index` manually, then retry `wi`\nhelp: run `wi doctor` to inspect setup",
            root.display()
        )
    })?;

    if let Some(message) = stats.reset_message {
        eprintln!("wi: {message}");
    }

    match index_is_fresh(root) {
        Ok(true) => Ok(()),
        Ok(false) => bail!(
            "index is still stale after one auto-build\nnext: run `build_index` manually in {}, inspect changed files, then retry `wi`\nhelp: run `wi doctor` to inspect setup",
            root.display()
        ),
        Err(error) => bail!(
            "index is not usable after one auto-build: {error:#}\nnext: run `build_index` manually in {}, then retry `wi`\nhelp: run `wi doctor` to inspect setup",
            root.display()
        ),
    }
}
