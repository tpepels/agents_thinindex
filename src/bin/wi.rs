use std::{collections::BTreeSet, env};

use anyhow::{Context, Result, bail};
use clap::Parser;
use thinindex::{
    bench::{BenchmarkRunOptions, render_benchmark_report, run_benchmark},
    binary_state::{ensure_binary_matches_source, print_version_if_requested},
    context::{render_impact_command, render_pack_command, render_refs_command},
    doctor::{render_doctor_report, run_doctor},
    indexer::{build_index, find_repo_root, index_is_fresh},
    search::{SearchOptions, format_file_result, format_result, search, search_files},
    stats::{self, UsageEvent},
    wi_cli::{WiArgs, WiCommand},
};

fn main() {
    if print_version_if_requested("wi") {
        return;
    }

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

    ensure_binary_matches_source(&root, "wi")?;
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
            let strong_record_paths = results
                .iter()
                .filter(|result| result.score <= 3)
                .map(|result| result.record.path.clone())
                .collect::<BTreeSet<_>>();
            let mut strong_results = results
                .iter()
                .filter(|result| result.score <= 3)
                .cloned()
                .collect::<Vec<_>>();
            strong_results.truncate(limit);

            let remaining_after_strong = limit.saturating_sub(strong_results.len());
            let mut file_results = if remaining_after_strong == 0 {
                Vec::new()
            } else {
                let mut file_options = options.clone();
                file_options.limit = remaining_after_strong;
                search_files(&root, &query, &file_options)?
            };
            file_results.retain(|result| !strong_record_paths.contains(&result.path));
            file_results.truncate(remaining_after_strong);

            let file_paths = file_results
                .iter()
                .map(|result| result.path.clone())
                .collect::<BTreeSet<_>>();
            let remaining_after_files =
                limit.saturating_sub(strong_results.len() + file_results.len());
            let mut weak_results = results
                .iter()
                .filter(|result| result.score > 3 && !file_paths.contains(&result.record.path))
                .cloned()
                .collect::<Vec<_>>();
            weak_results.truncate(remaining_after_files);

            let result_count = strong_results.len() + file_results.len() + weak_results.len();

            if result_count == 0 {
                print!("{}", render_search_miss(&query, &options));
            } else {
                for result in &strong_results {
                    println!("{}", format_result(result, options.verbose));
                }

                if !file_results.is_empty() {
                    if !strong_results.is_empty() {
                        println!();
                    }
                    println!("File matches:");
                    for result in &file_results {
                        println!("{}", format_file_result(result));
                    }

                    if !weak_results.is_empty() {
                        println!();
                        println!("Indexed symbols/content:");
                    }
                }

                for result in &weak_results {
                    println!("{}", format_result(result, options.verbose));
                }
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

fn render_search_miss(query: &str, options: &SearchOptions) -> String {
    let mut output = format!("No matches for: {}\n\nChecked:\n", query);

    if options.kind.is_none() && options.source.is_none() {
        output.push_str("- filenames/paths\n");
    }
    output.push_str("- indexed symbols/content\n\n");
    output.push_str("Try:\n");
    output.push_str("- rtk build_index\n");
    output.push_str("- rtk wi <filename-or-symbol>\n");
    output.push_str("- rtk wi <term> --path <path-substring>\n");
    output.push_str("- rtk wi refs <symbol>\n");

    output
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
