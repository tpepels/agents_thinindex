use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::store::open_ready_database;

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
    crate::store::indexed_file_count(root).unwrap_or_default()
}

pub fn append_usage_event(root: &Path, event: &UsageEvent) -> Result<()> {
    let conn = open_ready_database(root)?;

    conn.execute(
        "INSERT INTO usage_events(
            timestamp, query, query_len, result_count, hit,
            used_type, used_lang, used_path, used_limit, repo, indexed_files
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            i64::try_from(event.ts).context("usage timestamp is too large")?,
            &event.query,
            i64::try_from(event.query_len).context("usage query_len is too large")?,
            i64::try_from(event.result_count).context("usage result_count is too large")?,
            bool_to_i64(event.hit),
            bool_to_i64(event.used_type),
            bool_to_i64(event.used_lang),
            bool_to_i64(event.used_path),
            bool_to_i64(event.used_limit),
            &event.repo,
            i64::try_from(event.indexed_files).context("usage indexed_files is too large")?,
        ],
    )
    .context("failed to insert usage event")?;

    Ok(())
}

pub fn read_usage_events(root: &Path) -> Result<Vec<UsageEvent>> {
    let conn = open_ready_database(root)?;
    let mut stmt = conn
        .prepare(
            "SELECT timestamp, query, query_len, result_count, hit,
                    used_type, used_lang, used_path, used_limit, repo, indexed_files
             FROM usage_events
             ORDER BY id",
        )
        .context("failed to prepare usage event query")?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, i64>(10)?,
            ))
        })
        .context("failed to query usage events")?;

    let mut events = Vec::new();
    for row in rows {
        let (
            ts,
            query,
            query_len,
            result_count,
            hit,
            used_type,
            used_lang,
            used_path,
            used_limit,
            repo,
            indexed_files,
        ) = row.context("failed to read usage event row")?;

        events.push(UsageEvent {
            ts: u64::try_from(ts).context("usage timestamp must be non-negative")?,
            query,
            query_len: usize::try_from(query_len)
                .context("usage query_len must be non-negative")?,
            result_count: usize::try_from(result_count)
                .context("usage result_count must be non-negative")?,
            hit: hit != 0,
            used_type: used_type != 0,
            used_lang: used_lang != 0,
            used_path: used_path != 0,
            used_limit: used_limit != 0,
            repo,
            indexed_files: usize::try_from(indexed_files)
                .context("usage indexed_files must be non-negative")?,
        });
    }

    Ok(events)
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
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
        crate::store::prepare_for_build(root).expect("create sqlite index");

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
    fn read_usage_events_empty_database_is_empty() {
        let dir = tempdir().expect("tempdir");
        crate::store::prepare_for_build(dir.path()).expect("create sqlite index");
        let events = read_usage_events(dir.path()).expect("read");
        assert!(events.is_empty());
    }

    #[test]
    fn append_and_read_preserves_order() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        crate::store::prepare_for_build(root).expect("create sqlite index");

        let first = sample_event(1_700_000_000, true, 3);
        let second = sample_event(1_700_000_001, false, 0);
        append_usage_event(root, &first).expect("append first event");
        append_usage_event(root, &second).expect("append second event");

        let events = read_usage_events(root).expect("read");
        assert_eq!(events, vec![first, second]);
    }
}
