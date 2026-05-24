//! Escritura de snapshot Chromium en perfil Gecko (places.sqlite).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

use crate::{BookmarkEntry, HistoryEntry, ImporterError, ProfileSnapshot};

/// Superficies aplicables al perfil Gecko.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplySurface {
    Bookmarks,
    History,
}

/// Resultado de una aplicacion al perfil.
#[derive(Debug, Clone, Default)]
pub struct ApplyReport {
    pub bookmarks_added: usize,
    pub history_added: usize,
    pub backup_dir: Option<PathBuf>,
}

/// Carga un snapshot JSON exportado con `export_audit_snapshot`.
pub fn load_audit_snapshot(path: &Path) -> Result<ProfileSnapshot, ImporterError> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

/// Aplica marcadores e historial al perfil Gecko. Requiere navegador cerrado.
pub fn apply_snapshot_to_profile(
    profile_dir: &Path,
    snapshot: &ProfileSnapshot,
    surfaces: &[ApplySurface],
) -> Result<ApplyReport, ImporterError> {
    let places = profile_dir.join("places.sqlite");
    if !places.is_file() {
        return Err(ImporterError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "places.sqlite no existe: abre Aurexalis/Floorp una vez antes de import apply",
        )));
    }

    let backup_dir = backup_places(&places)?;
    let conn = Connection::open(&places)?;

    let mut report = ApplyReport {
        backup_dir: Some(backup_dir),
        ..ApplyReport::default()
    };

    if surfaces.contains(&ApplySurface::Bookmarks) {
        report.bookmarks_added = write_bookmarks(&conn, &snapshot.bookmarks)?;
    }
    if surfaces.contains(&ApplySurface::History) {
        report.history_added = write_history(&conn, &snapshot.history)?;
    }

    Ok(report)
}

fn backup_places(places: &Path) -> Result<PathBuf, ImporterError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup_dir = places
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("import-backups");
    fs::create_dir_all(&backup_dir)?;
    let dest = backup_dir.join(format!("places-{stamp}.sqlite"));
    fs::copy(places, &dest)?;
    Ok(backup_dir)
}

fn write_bookmarks(conn: &Connection, bookmarks: &[BookmarkEntry]) -> Result<usize, ImporterError> {
    let parent = ensure_unfiled_folder(conn)?;
    let now = now_micros();
    let mut added = 0usize;

    for bookmark in bookmarks {
        if bookmark.url.is_empty() {
            continue;
        }
        let place_id = upsert_place(conn, &bookmark.url, &bookmark.name, 0, None)?;
        let pos = next_child_position(conn, parent)?;
        conn.execute(
            "INSERT INTO moz_bookmarks (type, fk, parent, position, title, dateAdded, lastModified, guid)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?5, lower(hex(randomblob(8)) || hex(randomblob(4))))",
            params![1_i32, place_id, parent, pos, bookmark.name, now],
        )?;
        added += 1;
    }

    Ok(added)
}

fn write_history(conn: &Connection, history: &[HistoryEntry]) -> Result<usize, ImporterError> {
    let mut added = 0usize;
    for entry in history {
        if entry.url.is_empty() {
            continue;
        }
        let last_visit = chrome_time_to_firefox(entry.last_visit_time);
        let place_id = upsert_place(
            conn,
            &entry.url,
            &entry.title,
            entry.visit_count.max(1),
            Some(last_visit),
        )?;
        conn.execute(
            "INSERT INTO moz_historyvisits (place_id, visit_date, visit_type)
             VALUES (?1, ?2, 1)",
            params![place_id, last_visit],
        )?;
        added += 1;
    }
    Ok(added)
}

fn ensure_unfiled_folder(conn: &Connection) -> Result<i64, ImporterError> {
    let root: i64 = conn.query_row(
        "SELECT root FROM moz_bookmarks_roots WHERE root_name = 'unfiledRoot'",
        [],
        |row| row.get(0),
    )?;
    Ok(root)
}

fn upsert_place(
    conn: &Connection,
    url: &str,
    title: &str,
    visit_count: i64,
    last_visit: Option<i64>,
) -> Result<i64, ImporterError> {
    if let Ok(id) = conn.query_row(
        "SELECT id FROM moz_places WHERE url = ?1",
        [url],
        |row| row.get(0),
    ) {
        conn.execute(
            "UPDATE moz_places SET title = ?2, visit_count = MAX(visit_count, ?3),
             last_visit_date = COALESCE(?4, last_visit_date) WHERE id = ?1",
            params![id, title, visit_count, last_visit],
        )?;
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO moz_places (url, title, rev_host, visit_count, hidden, typed, frecency, last_visit_date, guid, foreign_count)
         VALUES (?1, ?2, '', ?3, 0, 0, -1, ?4, lower(hex(randomblob(8)) || hex(randomblob(4))), 0)",
        params![url, title, visit_count, last_visit],
    )?;
    Ok(conn.last_insert_rowid())
}

fn next_child_position(conn: &Connection, parent: i64) -> Result<i32, ImporterError> {
    let max_pos: Option<i32> = conn.query_row(
        "SELECT MAX(position) FROM moz_bookmarks WHERE parent = ?1",
        [parent],
        |row| row.get(0),
    )?;
    Ok(max_pos.unwrap_or(-1) + 1)
}

fn now_micros() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

fn chrome_time_to_firefox(chrome: i64) -> i64 {
    const EPOCH_DIFF: i64 = 11_644_473_600_000_000;
    if chrome > EPOCH_DIFF {
        chrome - EPOCH_DIFF
    } else {
        chrome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BookmarkEntry, HistoryEntry};

    fn minimal_places_db(path: &Path) {
        let conn = Connection::open(path).expect("open");
        conn.execute_batch(
            "
            CREATE TABLE moz_places (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              url TEXT, title TEXT, rev_host TEXT,
              visit_count INTEGER DEFAULT 0,
              hidden INTEGER DEFAULT 0, typed INTEGER DEFAULT 0,
              frecency INTEGER DEFAULT -1, last_visit_date INTEGER,
              guid TEXT, foreign_count INTEGER DEFAULT 0
            );
            CREATE TABLE moz_bookmarks (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              type INTEGER, fk INTEGER, parent INTEGER, position INTEGER,
              title TEXT, dateAdded INTEGER, lastModified INTEGER, guid TEXT
            );
            CREATE TABLE moz_bookmarks_roots (root_name TEXT PRIMARY KEY, root INTEGER);
            CREATE TABLE moz_historyvisits (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              place_id INTEGER, visit_date INTEGER, visit_type INTEGER
            );
            INSERT INTO moz_bookmarks (id, type, parent, position, title, dateAdded, lastModified)
              VALUES (5, 2, 0, 0, 'Unfiled', 0, 0);
            INSERT INTO moz_bookmarks_roots VALUES ('unfiledRoot', 5);
            ",
        )
        .expect("schema");
    }

    #[test]
    fn applies_bookmarks_and_history() {
        let dir = std::env::temp_dir().join("aurexalis-gecko-write-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        minimal_places_db(&dir.join("places.sqlite"));

        let snapshot = ProfileSnapshot {
            cookies: vec![],
            logins: vec![],
            history: vec![HistoryEntry {
                url: "https://example.com/".to_owned(),
                title: "Example".to_owned(),
                visit_count: 2,
                typed_count: 0,
                last_visit_time: 1_300_000_000_000_000,
            }],
            favicons: vec![],
            bookmarks: vec![BookmarkEntry {
                name: "Example".to_owned(),
                url: "https://example.com/".to_owned(),
                path: vec![],
            }],
            preferences: None,
            secure_preferences: None,
            local_state: None,
        };

        let report = apply_snapshot_to_profile(
            &dir,
            &snapshot,
            &[ApplySurface::Bookmarks, ApplySurface::History],
        )
        .expect("apply");

        assert_eq!(report.bookmarks_added, 1);
        assert_eq!(report.history_added, 1);

        let conn = Connection::open(dir.join("places.sqlite")).expect("open");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM moz_places", [], |r| r.get(0))
            .expect("count");
        assert_eq!(count, 1);

        let _ = fs::remove_dir_all(dir);
    }
}
