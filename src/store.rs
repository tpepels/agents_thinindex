use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};
use rusqlite::{Connection, OptionalExtension, params};

use crate::model::{
    DependencyEdge, FileMeta, INDEX_SCHEMA_VERSION, IndexRecord, Manifest, ReferenceRecord,
    SemanticFact, SemanticFactKind,
};

pub const DEV_INDEX_DIR: &str = ".dev_index";
pub const SQLITE_FILE: &str = "index.sqlite";

pub fn index_dir(root: &Path) -> PathBuf {
    root.join(DEV_INDEX_DIR)
}

pub fn sqlite_path(root: &Path) -> PathBuf {
    index_dir(root).join(SQLITE_FILE)
}

pub fn ensure_index_dir(root: &Path) -> Result<()> {
    let path = index_dir(root);

    fs::create_dir_all(&path)
        .with_context(|| format!("failed to create index directory: {}", path.display()))?;

    Ok(())
}

/// Removes the entire .dev_index directory and recreates it.
pub fn reset_dev_index(root: &Path) -> Result<()> {
    let dir = index_dir(root);
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("failed to remove {}", dir.display()))?;
    }
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(())
}

pub fn prepare_for_build(root: &Path) -> Result<(Manifest, Option<&'static str>)> {
    if old_jsonl_storage_exists(root) {
        reset_dev_index(root)?;
        let conn = create_database(root)?;
        return Ok((
            load_manifest_from_conn(&conn)?,
            Some("old index storage found; rebuilding .dev_index"),
        ));
    }

    let path = sqlite_path(root);
    if !path.exists() {
        let conn = create_database(root)?;
        return Ok((load_manifest_from_conn(&conn)?, None));
    }

    match open_existing_database(root).and_then(|conn| {
        validate_schema_version(&conn)?;
        load_manifest_from_conn(&conn)
    }) {
        Ok(manifest) => Ok((manifest, None)),
        Err(error) => {
            let message = if schema_version_mismatch(&error) {
                "index schema changed; rebuilding .dev_index"
            } else {
                "index database invalid; rebuilding .dev_index"
            };
            reset_dev_index(root)?;
            let conn = create_database(root)?;
            Ok((load_manifest_from_conn(&conn)?, Some(message)))
        }
    }
}

pub fn load_manifest(root: &Path) -> Result<Manifest> {
    let conn = open_ready_database(root)?;
    load_manifest_from_conn(&conn)
}

pub fn load_records(root: &Path) -> Result<Vec<IndexRecord>> {
    let conn = open_ready_database(root)?;
    load_records_from_conn(&conn)
}

pub fn load_refs(root: &Path) -> Result<Vec<ReferenceRecord>> {
    let conn = open_ready_database(root)?;
    load_refs_from_conn(&conn)
}

pub fn load_dependencies(root: &Path) -> Result<Vec<DependencyEdge>> {
    let conn = open_ready_database(root)?;
    load_dependencies_from_conn(&conn)
}

pub fn load_semantic_facts(root: &Path) -> Result<Vec<SemanticFact>> {
    let conn = open_ready_database(root)?;
    load_semantic_facts_from_conn(&conn)
}

pub fn save_index_snapshot(
    root: &Path,
    manifest: &Manifest,
    records: &[IndexRecord],
    refs: &[ReferenceRecord],
    dependencies: &[DependencyEdge],
    semantic_facts: &[SemanticFact],
) -> Result<()> {
    let mut conn = open_ready_database(root)?;
    let tx = conn.transaction().with_context(|| {
        format!(
            "failed to begin transaction: {}",
            sqlite_path(root).display()
        )
    })?;

    tx.execute("DELETE FROM records", [])
        .context("failed to clear records table")?;
    tx.execute("DELETE FROM refs", [])
        .context("failed to clear refs table")?;
    tx.execute("DELETE FROM dependencies", [])
        .context("failed to clear dependencies table")?;
    tx.execute("DELETE FROM semantic_facts", [])
        .context("failed to clear semantic facts table")?;
    tx.execute("DELETE FROM files", [])
        .context("failed to clear files table")?;

    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO files(path, mtime_ns, size)
                 VALUES (?1, ?2, ?3)",
            )
            .context("failed to prepare file insert")?;

        for (path, meta) in &manifest.files {
            stmt.execute(params![
                path,
                i64_from_u128(meta.mtime_ns, "mtime_ns")?,
                i64_from_u64(meta.size, "size")?,
            ])
            .with_context(|| format!("failed to insert file metadata for {path}"))?;
        }
    }

    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO records(path, line, col, lang, kind, name, text, source)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .context("failed to prepare record insert")?;

        for record in records {
            stmt.execute(params![
                record.path,
                i64_from_usize(record.line, "line")?,
                i64_from_usize(record.col, "col")?,
                record.lang,
                record.kind,
                record.name,
                record.text,
                record.source,
            ])
            .with_context(|| {
                format!(
                    "failed to insert index record at {}:{}:{}",
                    record.path, record.line, record.col
                )
            })?;
        }
    }

    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO refs(
                    from_path,
                    from_line,
                    from_col,
                    to_name,
                    to_kind,
                    ref_kind,
                    confidence,
                    reason,
                    evidence,
                    source
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )
            .context("failed to prepare ref insert")?;

        for reference in refs {
            stmt.execute(params![
                &reference.from_path,
                i64_from_usize(reference.from_line, "from_line")?,
                i64_from_usize(reference.from_col, "from_col")?,
                &reference.to_name,
                reference.to_kind.as_deref(),
                &reference.ref_kind,
                &reference.confidence,
                reference.reason.as_deref(),
                &reference.evidence,
                &reference.source,
            ])
            .with_context(|| {
                format!(
                    "failed to insert reference at {}:{}:{} to {}",
                    reference.from_path, reference.from_line, reference.from_col, reference.to_name
                )
            })?;
        }
    }

    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO dependencies(
                    from_path,
                    from_line,
                    from_col,
                    import_path,
                    target_path,
                    dependency_kind,
                    lang,
                    confidence,
                    unresolved_reason,
                    evidence,
                    source
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )
            .context("failed to prepare dependency insert")?;

        for dependency in dependencies {
            stmt.execute(params![
                &dependency.from_path,
                i64_from_usize(dependency.from_line, "from_line")?,
                i64_from_usize(dependency.from_col, "from_col")?,
                &dependency.import_path,
                dependency.target_path.as_deref(),
                &dependency.dependency_kind,
                &dependency.lang,
                &dependency.confidence,
                dependency.unresolved_reason.as_deref(),
                &dependency.evidence,
                &dependency.source,
            ])
            .with_context(|| {
                format!(
                    "failed to insert dependency at {}:{}:{} to {}",
                    dependency.from_path,
                    dependency.from_line,
                    dependency.from_col,
                    dependency.import_path
                )
            })?;
        }
    }

    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO semantic_facts(
                    source_path,
                    source_line,
                    source_col,
                    kind,
                    symbol,
                    target_path,
                    target_line,
                    target_col,
                    detail,
                    confidence,
                    adapter
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )
            .context("failed to prepare semantic fact insert")?;

        for fact in semantic_facts {
            stmt.execute(params![
                &fact.source_path,
                i64_from_usize(fact.source_line, "source_line")?,
                i64_from_usize(fact.source_col, "source_col")?,
                fact.kind.as_str(),
                &fact.symbol,
                fact.target_path.as_deref(),
                option_i64_from_usize(fact.target_line, "target_line")?,
                option_i64_from_usize(fact.target_col, "target_col")?,
                fact.detail.as_deref(),
                &fact.confidence,
                &fact.adapter,
            ])
            .with_context(|| {
                format!(
                    "failed to insert semantic fact at {}:{}:{} for {}",
                    fact.source_path, fact.source_line, fact.source_col, fact.symbol
                )
            })?;
        }
    }

    upsert_meta(&tx, "schema_version", &INDEX_SCHEMA_VERSION.to_string())?;
    upsert_meta(&tx, "updated_at", &now_unix_seconds().to_string())?;

    tx.commit()
        .with_context(|| format!("failed to commit {}", sqlite_path(root).display()))?;

    Ok(())
}

pub fn indexed_file_count(root: &Path) -> Result<usize> {
    let conn = open_ready_database(root)?;
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
        .context("failed to count indexed files")?;

    usize_from_i64(count, "file count")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexCounts {
    pub records: usize,
    pub refs: usize,
    pub dependencies: usize,
    pub semantic_facts: usize,
}

pub fn load_index_counts(root: &Path) -> Result<IndexCounts> {
    let conn = open_ready_database(root)?;

    Ok(IndexCounts {
        records: table_count(&conn, "records")?,
        refs: table_count(&conn, "refs")?,
        dependencies: table_count(&conn, "dependencies")?,
        semantic_facts: table_count(&conn, "semantic_facts")?,
    })
}

pub fn open_ready_database(root: &Path) -> Result<Connection> {
    let conn = open_existing_database(root)?;
    validate_schema_version(&conn)?;
    Ok(conn)
}

pub fn old_jsonl_storage_exists(root: &Path) -> bool {
    let dir = index_dir(root);
    ["manifest.json", "index.jsonl", "wi_usage.jsonl"]
        .iter()
        .any(|name| dir.join(name).exists())
}

fn create_database(root: &Path) -> Result<Connection> {
    ensure_index_dir(root)?;
    let path = sqlite_path(root);
    let conn =
        Connection::open(&path).with_context(|| format!("failed to open {}", path.display()))?;
    configure_connection(&conn)?;
    initialize_schema(&conn)?;
    Ok(conn)
}

fn open_existing_database(root: &Path) -> Result<Connection> {
    if old_jsonl_storage_exists(root) && !sqlite_path(root).exists() {
        bail!("old JSONL index storage found; run `build_index`");
    }

    let path = sqlite_path(root);
    if !path.exists() {
        bail!("index database missing; run `build_index`");
    }

    let conn = Connection::open(&path)
        .with_context(|| format!("failed to open {}; run `build_index`", path.display()))?;
    configure_connection(&conn)?;
    Ok(conn)
}

fn configure_connection(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA temp_store = MEMORY;
        PRAGMA cache_size = -20000;
        ",
    )
    .context("failed to configure SQLite connection")?;

    Ok(())
}

fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS files (
            path TEXT PRIMARY KEY,
            mtime_ns INTEGER NOT NULL,
            size INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS records (
            path TEXT NOT NULL,
            line INTEGER NOT NULL,
            col INTEGER NOT NULL,
            lang TEXT NOT NULL,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            text TEXT NOT NULL,
            source TEXT NOT NULL,
            UNIQUE(path, line, col)
        );
        CREATE INDEX IF NOT EXISTS records_name_idx ON records(name);
        CREATE INDEX IF NOT EXISTS records_kind_idx ON records(kind);
        CREATE INDEX IF NOT EXISTS records_lang_idx ON records(lang);
        CREATE INDEX IF NOT EXISTS records_path_idx ON records(path);
        CREATE INDEX IF NOT EXISTS records_source_idx ON records(source);
        CREATE TABLE IF NOT EXISTS refs (
            from_path TEXT NOT NULL,
            from_line INTEGER NOT NULL,
            from_col INTEGER NOT NULL,
            to_name TEXT NOT NULL,
            to_kind TEXT,
            ref_kind TEXT NOT NULL,
            confidence TEXT NOT NULL,
            reason TEXT,
            evidence TEXT NOT NULL,
            source TEXT NOT NULL,
            UNIQUE(from_path, from_line, from_col, to_name, ref_kind)
        );
        CREATE INDEX IF NOT EXISTS refs_to_name_idx ON refs(to_name);
        CREATE INDEX IF NOT EXISTS refs_from_path_idx ON refs(from_path);
        CREATE INDEX IF NOT EXISTS refs_ref_kind_idx ON refs(ref_kind);
        CREATE INDEX IF NOT EXISTS refs_confidence_idx ON refs(confidence);
        CREATE INDEX IF NOT EXISTS refs_source_idx ON refs(source);
        CREATE TABLE IF NOT EXISTS dependencies (
            from_path TEXT NOT NULL,
            from_line INTEGER NOT NULL,
            from_col INTEGER NOT NULL,
            import_path TEXT NOT NULL,
            target_path TEXT,
            dependency_kind TEXT NOT NULL,
            lang TEXT NOT NULL,
            confidence TEXT NOT NULL,
            unresolved_reason TEXT,
            evidence TEXT NOT NULL,
            source TEXT NOT NULL,
            UNIQUE(from_path, import_path, target_path, dependency_kind)
        );
        CREATE INDEX IF NOT EXISTS dependencies_from_path_idx ON dependencies(from_path);
        CREATE INDEX IF NOT EXISTS dependencies_target_path_idx ON dependencies(target_path);
        CREATE INDEX IF NOT EXISTS dependencies_import_path_idx ON dependencies(import_path);
        CREATE INDEX IF NOT EXISTS dependencies_kind_idx ON dependencies(dependency_kind);
        CREATE INDEX IF NOT EXISTS dependencies_lang_idx ON dependencies(lang);
        CREATE INDEX IF NOT EXISTS dependencies_confidence_idx ON dependencies(confidence);
        CREATE TABLE IF NOT EXISTS semantic_facts (
            source_path TEXT NOT NULL,
            source_line INTEGER NOT NULL,
            source_col INTEGER NOT NULL,
            kind TEXT NOT NULL,
            symbol TEXT NOT NULL,
            target_path TEXT,
            target_line INTEGER,
            target_col INTEGER,
            detail TEXT,
            confidence TEXT NOT NULL,
            adapter TEXT NOT NULL,
            UNIQUE(source_path, source_line, source_col, kind, symbol, target_path, adapter)
        );
        CREATE INDEX IF NOT EXISTS semantic_facts_source_path_idx ON semantic_facts(source_path);
        CREATE INDEX IF NOT EXISTS semantic_facts_target_path_idx ON semantic_facts(target_path);
        CREATE INDEX IF NOT EXISTS semantic_facts_symbol_idx ON semantic_facts(symbol);
        CREATE INDEX IF NOT EXISTS semantic_facts_kind_idx ON semantic_facts(kind);
        CREATE INDEX IF NOT EXISTS semantic_facts_adapter_idx ON semantic_facts(adapter);
        CREATE TABLE IF NOT EXISTS usage_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            command TEXT NOT NULL,
            query TEXT NOT NULL,
            query_len INTEGER NOT NULL,
            result_count INTEGER NOT NULL,
            hit INTEGER NOT NULL,
            used_type INTEGER NOT NULL,
            used_lang INTEGER NOT NULL,
            used_path INTEGER NOT NULL,
            used_limit INTEGER NOT NULL,
            repo TEXT NOT NULL,
            indexed_files INTEGER NOT NULL
        );
        ",
    )
    .context("failed to initialize SQLite schema")?;

    let now = now_unix_seconds().to_string();
    upsert_meta(conn, "schema_version", &INDEX_SCHEMA_VERSION.to_string())?;
    upsert_meta(conn, "created_at", &now)?;
    upsert_meta(conn, "updated_at", &now)?;

    Ok(())
}

fn validate_schema_version(conn: &Connection) -> Result<()> {
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .optional()
        .context("failed to read index schema version; run `build_index`")?;

    let Some(value) = value else {
        bail!("index schema version missing; run `build_index`");
    };

    let version = value.parse::<u32>().with_context(|| {
        format!("index schema version is invalid ({value:?}); run `build_index`")
    })?;

    if version != INDEX_SCHEMA_VERSION {
        bail!(
            "index schema version {version} does not match {INDEX_SCHEMA_VERSION}; run `build_index`"
        );
    }

    Ok(())
}

fn schema_version_mismatch(error: &anyhow::Error) -> bool {
    let text = format!("{error:#}");
    text.contains("does not match")
}

fn table_count(conn: &Connection, table: &str) -> Result<usize> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    let count: i64 = conn
        .query_row(&sql, [], |row| row.get(0))
        .with_context(|| format!("failed to count {table} rows"))?;

    usize_from_i64(count, table)
}

fn upsert_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO meta(key, value)
         VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .with_context(|| format!("failed to write meta key {key}"))?;

    Ok(())
}

fn load_manifest_from_conn(conn: &Connection) -> Result<Manifest> {
    let mut stmt = conn
        .prepare("SELECT path, mtime_ns, size FROM files ORDER BY path")
        .context("failed to prepare file metadata query")?;

    let mut rows = stmt.query([]).context("failed to query file metadata")?;
    let mut files = BTreeMap::new();

    while let Some(row) = rows.next().context("failed to read file metadata row")? {
        let path: String = row.get(0).context("failed to read file path")?;
        let mtime_ns: i64 = row.get(1).context("failed to read file mtime")?;
        let size: i64 = row.get(2).context("failed to read file size")?;

        files.insert(
            path,
            FileMeta {
                mtime_ns: u128_from_i64(mtime_ns, "mtime_ns")?,
                size: u64_from_i64(size, "size")?,
            },
        );
    }

    Ok(Manifest {
        schema_version: INDEX_SCHEMA_VERSION,
        files,
    })
}

fn load_records_from_conn(conn: &Connection) -> Result<Vec<IndexRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT path, line, col, lang, kind, name, text, source
             FROM records
             ORDER BY path, line, col, kind, name, source",
        )
        .context("failed to prepare record query")?;

    let rows = stmt
        .query_map([], |row| {
            let line: i64 = row.get(1)?;
            let col: i64 = row.get(2)?;

            Ok((
                row.get::<_, String>(0)?,
                line,
                col,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .context("failed to query records")?;

    let mut records = Vec::new();
    for row in rows {
        let (path, line, col, lang, kind, name, text, source) =
            row.context("failed to read record row")?;

        records.push(IndexRecord {
            path,
            line: usize_from_i64(line, "line")?,
            col: usize_from_i64(col, "col")?,
            lang,
            kind,
            name,
            text,
            source,
        });
    }

    Ok(records)
}

fn load_refs_from_conn(conn: &Connection) -> Result<Vec<ReferenceRecord>> {
    let mut stmt = conn
        .prepare(
            "SELECT
                from_path,
                from_line,
                from_col,
                to_name,
                to_kind,
                ref_kind,
                confidence,
                reason,
                evidence,
                source
             FROM refs
             ORDER BY from_path, from_line, from_col, to_name, ref_kind, source",
        )
        .context("failed to prepare refs query")?;

    let rows = stmt
        .query_map([], |row| {
            let from_line: i64 = row.get(1)?;
            let from_col: i64 = row.get(2)?;

            Ok((
                row.get::<_, String>(0)?,
                from_line,
                from_col,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
            ))
        })
        .context("failed to query refs")?;

    let mut refs = Vec::new();
    for row in rows {
        let (
            from_path,
            from_line,
            from_col,
            to_name,
            to_kind,
            ref_kind,
            confidence,
            reason,
            evidence,
            source,
        ) = row.context("failed to read ref row")?;

        refs.push(ReferenceRecord {
            from_path,
            from_line: usize_from_i64(from_line, "from_line")?,
            from_col: usize_from_i64(from_col, "from_col")?,
            to_name,
            to_kind,
            ref_kind,
            confidence,
            reason,
            evidence,
            source,
        });
    }

    Ok(refs)
}

fn load_dependencies_from_conn(conn: &Connection) -> Result<Vec<DependencyEdge>> {
    let mut stmt = conn
        .prepare(
            "SELECT
                from_path,
                from_line,
                from_col,
                import_path,
                target_path,
                dependency_kind,
                lang,
                confidence,
                unresolved_reason,
                evidence,
                source
             FROM dependencies
             ORDER BY from_path, from_line, from_col, import_path, dependency_kind, source",
        )
        .context("failed to prepare dependency query")?;

    let rows = stmt
        .query_map([], |row| {
            let from_line: i64 = row.get(1)?;
            let from_col: i64 = row.get(2)?;

            Ok((
                row.get::<_, String>(0)?,
                from_line,
                from_col,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })
        .context("failed to query dependencies")?;

    let mut dependencies = Vec::new();
    for row in rows {
        let (
            from_path,
            from_line,
            from_col,
            import_path,
            target_path,
            dependency_kind,
            lang,
            confidence,
            unresolved_reason,
            evidence,
            source,
        ) = row.context("failed to read dependency row")?;

        dependencies.push(DependencyEdge {
            from_path,
            from_line: usize_from_i64(from_line, "from_line")?,
            from_col: usize_from_i64(from_col, "from_col")?,
            import_path,
            target_path,
            dependency_kind,
            lang,
            confidence,
            unresolved_reason,
            evidence,
            source,
        });
    }

    Ok(dependencies)
}

fn load_semantic_facts_from_conn(conn: &Connection) -> Result<Vec<SemanticFact>> {
    let mut stmt = conn
        .prepare(
            "SELECT
                source_path,
                source_line,
                source_col,
                kind,
                symbol,
                target_path,
                target_line,
                target_col,
                detail,
                confidence,
                adapter
             FROM semantic_facts
             ORDER BY source_path, source_line, source_col, kind, symbol, target_path, adapter",
        )
        .context("failed to prepare semantic facts query")?;

    let rows = stmt
        .query_map([], |row| {
            let source_line: i64 = row.get(1)?;
            let source_col: i64 = row.get(2)?;
            let target_line: Option<i64> = row.get(6)?;
            let target_col: Option<i64> = row.get(7)?;

            Ok((
                row.get::<_, String>(0)?,
                source_line,
                source_col,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                target_line,
                target_col,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, String>(10)?,
            ))
        })
        .context("failed to query semantic facts")?;

    let mut facts = Vec::new();
    for row in rows {
        let (
            source_path,
            source_line,
            source_col,
            kind,
            symbol,
            target_path,
            target_line,
            target_col,
            detail,
            confidence,
            adapter,
        ) = row.context("failed to read semantic fact row")?;
        let kind = kind
            .parse::<SemanticFactKind>()
            .map_err(|error| anyhow!("{error}"))?;

        facts.push(SemanticFact {
            source_path,
            source_line: usize_from_i64(source_line, "source_line")?,
            source_col: usize_from_i64(source_col, "source_col")?,
            kind,
            symbol,
            target_path,
            target_line: option_usize_from_i64(target_line, "target_line")?,
            target_col: option_usize_from_i64(target_col, "target_col")?,
            detail,
            confidence,
            adapter,
        });
    }

    Ok(facts)
}

pub fn remove_records_for_paths(records: Vec<IndexRecord>, paths: &[String]) -> Vec<IndexRecord> {
    records
        .into_iter()
        .filter(|record| !paths.contains(&record.path))
        .collect()
}

pub fn remove_refs_for_paths(refs: Vec<ReferenceRecord>, paths: &[String]) -> Vec<ReferenceRecord> {
    refs.into_iter()
        .filter(|reference| !paths.contains(&reference.from_path))
        .collect()
}

pub fn sort_records(records: &mut [IndexRecord]) {
    records.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.cmp(&b.line))
            .then(a.col.cmp(&b.col))
            .then(a.kind.cmp(&b.kind))
            .then(a.name.cmp(&b.name))
            .then(a.source.cmp(&b.source))
    });
}

pub fn sort_refs(refs: &mut [ReferenceRecord]) {
    refs.sort_by(|a, b| {
        a.from_path
            .cmp(&b.from_path)
            .then(a.from_line.cmp(&b.from_line))
            .then(a.from_col.cmp(&b.from_col))
            .then(a.to_name.cmp(&b.to_name))
            .then(a.ref_kind.cmp(&b.ref_kind))
            .then(a.source.cmp(&b.source))
    });
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn i64_from_u128(value: u128, label: &str) -> Result<i64> {
    i64::try_from(value).map_err(|_| anyhow!("{label} is too large for SQLite INTEGER: {value}"))
}

fn i64_from_u64(value: u64, label: &str) -> Result<i64> {
    i64::try_from(value).map_err(|_| anyhow!("{label} is too large for SQLite INTEGER: {value}"))
}

fn i64_from_usize(value: usize, label: &str) -> Result<i64> {
    i64::try_from(value).map_err(|_| anyhow!("{label} is too large for SQLite INTEGER: {value}"))
}

fn option_i64_from_usize(value: Option<usize>, label: &str) -> Result<Option<i64>> {
    value.map(|value| i64_from_usize(value, label)).transpose()
}

fn u128_from_i64(value: i64, label: &str) -> Result<u128> {
    u128::try_from(value).map_err(|_| anyhow!("{label} must be non-negative, got {value}"))
}

fn u64_from_i64(value: i64, label: &str) -> Result<u64> {
    u64::try_from(value).map_err(|_| anyhow!("{label} must be non-negative, got {value}"))
}

fn usize_from_i64(value: i64, label: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| anyhow!("{label} must be non-negative, got {value}"))
}

fn option_usize_from_i64(value: Option<i64>, label: &str) -> Result<Option<usize>> {
    value.map(|value| usize_from_i64(value, label)).transpose()
}
