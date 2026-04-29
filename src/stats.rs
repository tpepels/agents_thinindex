use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::store::{ensure_index_dir, index_dir};

pub const USAGE_FILE: &str = "wi_usage.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UsageEvent {
    pub ts: u64,
    pub query: String,
    pub query_len: usize,
    pub result_count: usize,
    pub hit: bool,
    pub used_type: bool,
    pub used_lang: bool,
    pub used_path: bool,
    pub used_limit: bool,
    pub repo: String,
    pub indexed_files: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowStats {
    pub label: &'static str,
    pub days: u64,
    pub total: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_ratio: f64,
    pub avg_results: f64,
}

pub const WINDOW_DAYS: &[(u64, &str)] = &[(1, "1d"), (2, "2d"), (5, "5d"), (30, "30d")];

pub fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn manifest_indexed_files(root: &Path) -> usize {
    match crate::store::load_manifest(root) {
        Ok(manifest) => manifest.files.len(),
        Err(_) => 0,
    }
}

pub fn append_usage_event(root: &Path, event: &UsageEvent) -> Result<()> {
    ensure_index_dir(root)?;

    let path = index_dir(root).join(USAGE_FILE);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open usage log: {}", path.display()))?;

    let mut line = serde_json::to_vec(event).context("failed to serialize usage event")?;
    line.push(b'\n');

    // Single write_all so concurrent `wi` invocations cannot interleave bytes
    // mid-record under O_APPEND.
    file.write_all(&line)
        .with_context(|| format!("failed to write usage log: {}", path.display()))?;

    Ok(())
}

pub fn read_usage_events(root: &Path) -> Result<Vec<UsageEvent>> {
    let path = index_dir(root).join(USAGE_FILE);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(&path)
        .with_context(|| format!("failed to read usage log: {}", path.display()))?;

    let mut events = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<UsageEvent>(line) {
            Ok(event) => events.push(event),
            Err(error) => eprintln!(
                "warning: skipping malformed usage event on line {} of {}: {error}",
                idx + 1,
                path.display()
            ),
        }
    }

    Ok(events)
}

pub fn compute_windows(events: &[UsageEvent], now: u64) -> Vec<WindowStats> {
    WINDOW_DAYS
        .iter()
        .map(|(days, label)| {
            let cutoff = now.saturating_sub(days * 86_400);

            let mut total = 0usize;
            let mut hits = 0usize;
            let mut result_sum = 0usize;

            for event in events.iter().filter(|e| e.ts >= cutoff) {
                total += 1;
                if event.hit {
                    hits += 1;
                }
                result_sum += event.result_count;
            }

            let misses = total - hits;
            let hit_ratio = if total == 0 {
                0.0
            } else {
                hits as f64 / total as f64
            };
            let avg_results = if total == 0 {
                0.0
            } else {
                result_sum as f64 / total as f64
            };

            WindowStats {
                label,
                days: *days,
                total,
                hits,
                misses,
                hit_ratio,
                avg_results,
            }
        })
        .collect()
}

pub fn ascii_bar(value: usize, max_value: usize, max_width: usize) -> String {
    if max_value == 0 || value == 0 {
        return String::new();
    }

    let scaled = (value as f64 / max_value as f64) * max_width as f64;
    let mut width = scaled.round() as usize;

    if width == 0 {
        width = 1;
    }

    if width > max_width {
        width = max_width;
    }

    "#".repeat(width)
}

pub fn render_usage_table(windows: &[WindowStats]) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "{:<10}{:<10}{:<6}{:<8}{:<11}{}\n",
        "Window", "Searches", "Hits", "Misses", "Hit ratio", "Avg results"
    ));

    for window in windows {
        let ratio = format!("{:.1}%", window.hit_ratio * 100.0);
        let avg = format!("{:.1}", window.avg_results);

        out.push_str(&format!(
            "{:<10}{:<10}{:<6}{:<8}{:<11}{}\n",
            window.label, window.total, window.hits, window.misses, ratio, avg
        ));
    }

    out
}

pub fn render_hit_miss_graph(windows: &[WindowStats]) -> String {
    let max_value = windows
        .iter()
        .flat_map(|w| [w.hits, w.misses])
        .max()
        .unwrap_or(0);

    let max_width = 30;
    let mut out = String::from("Hit/miss graph\n");

    for window in windows {
        let hits_bar = ascii_bar(window.hits, max_value, max_width);
        let misses_bar = ascii_bar(window.misses, max_value, max_width);

        out.push_str(&format!(
            "{:<5}H {} {:<4}  M {} {}\n",
            window.label, hits_bar, window.hits, misses_bar, window.misses
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_event(ts: u64, hit: bool, result_count: usize) -> UsageEvent {
        UsageEvent {
            ts,
            query: "needle".to_string(),
            query_len: 6,
            result_count,
            hit,
            used_type: false,
            used_lang: false,
            used_path: false,
            used_limit: false,
            repo: "/tmp/repo".to_string(),
            indexed_files: 4,
        }
    }

    #[test]
    fn ascii_bar_zero_value_is_empty() {
        assert_eq!(ascii_bar(0, 10, 30), "");
    }

    #[test]
    fn ascii_bar_full_width() {
        assert_eq!(ascii_bar(10, 10, 30), "#".repeat(30));
    }

    #[test]
    fn ascii_bar_half_width() {
        assert_eq!(ascii_bar(5, 10, 30), "#".repeat(15));
    }

    #[test]
    fn ascii_bar_floor_at_one() {
        let bar = ascii_bar(1, 100, 30);
        assert!(!bar.is_empty());
        assert!(bar.chars().all(|c| c == '#'));
    }

    #[test]
    fn ascii_bar_zero_max_value_is_empty() {
        assert_eq!(ascii_bar(7, 0, 30), "");
    }

    #[test]
    fn compute_windows_filters_by_timestamp() {
        let now = 10 * 86_400;
        let events = vec![
            sample_event(now - 10, true, 1),
            sample_event(now - 2 * 86_400, true, 2),
            sample_event(now - 10 * 86_400, true, 3),
        ];

        let windows = compute_windows(&events, now);

        let by_label = |label: &str| {
            windows
                .iter()
                .find(|w| w.label == label)
                .expect("window present")
                .clone()
        };

        assert_eq!(by_label("1d").total, 1);
        assert_eq!(by_label("2d").total, 2);
        assert_eq!(by_label("5d").total, 2);
        assert_eq!(by_label("30d").total, 3);
    }

    #[test]
    fn compute_windows_zero_for_empty_events() {
        let windows = compute_windows(&[], 1_000_000);

        for window in windows {
            assert_eq!(window.total, 0);
            assert_eq!(window.hits, 0);
            assert_eq!(window.misses, 0);
            assert_eq!(window.hit_ratio, 0.0);
            assert_eq!(window.avg_results, 0.0);
        }
    }

    #[test]
    fn compute_windows_known_mix() {
        let now = 1_000_000u64;
        let events = vec![
            sample_event(now - 1, true, 2),
            sample_event(now - 2, true, 4),
            sample_event(now - 3, true, 6),
            sample_event(now - 4, false, 0),
        ];

        let windows = compute_windows(&events, now);
        let one_day = windows
            .iter()
            .find(|w| w.label == "1d")
            .expect("1d window")
            .clone();

        assert_eq!(one_day.total, 4);
        assert_eq!(one_day.hits, 3);
        assert_eq!(one_day.misses, 1);
        assert!((one_day.hit_ratio - 0.75).abs() < 1e-9);
        assert!((one_day.avg_results - 3.0).abs() < 1e-9);
    }

    #[test]
    fn append_and_read_round_trip() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let event = UsageEvent {
            ts: 1_700_000_000,
            query: "Foo".to_string(),
            query_len: 3,
            result_count: 2,
            hit: true,
            used_type: true,
            used_lang: false,
            used_path: false,
            used_limit: true,
            repo: root.display().to_string(),
            indexed_files: 7,
        };

        append_usage_event(root, &event).expect("append");
        let events = read_usage_events(root).expect("read");

        assert_eq!(events, vec![event]);
    }

    #[test]
    fn read_usage_events_missing_file_is_empty() {
        let dir = tempdir().expect("tempdir");
        let events = read_usage_events(dir.path()).expect("read");
        assert!(events.is_empty());
    }

    #[test]
    fn read_usage_events_skips_malformed_lines() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();

        let good = sample_event(1_700_000_000, true, 3);
        append_usage_event(root, &good).expect("append good event");

        // Simulate a corrupted line from a concurrent-append interleave: two
        // valid JSON objects glued together with no newline between them.
        let path = index_dir(root).join(USAGE_FILE);
        let glued = format!(
            "{}{}\n",
            serde_json::to_string(&good).unwrap(),
            serde_json::to_string(&good).unwrap(),
        );
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(glued.as_bytes()).unwrap();

        append_usage_event(root, &good).expect("append second good event");

        let events = read_usage_events(root).expect("read");
        assert_eq!(events, vec![good.clone(), good]);
    }
}
