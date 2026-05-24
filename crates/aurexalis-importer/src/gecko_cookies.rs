//! Escritura de cookies Chromium en `cookies.sqlite` del perfil Gecko.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

use crate::{CookieRecord, ImporterError, SecretValue};

/// Escribe cookies descifradas en `cookies.sqlite`. Requiere navegador cerrado.
pub fn write_cookies(profile_dir: &Path, cookies: &[CookieRecord]) -> Result<CookieWriteReport, ImporterError> {
    let cookies_db = profile_dir.join("cookies.sqlite");
    if !cookies_db.is_file() {
        return Err(ImporterError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "cookies.sqlite no existe: abre Aurexalis una vez antes de import apply --cookies-only",
        )));
    }

    let backup_dir = backup_file(&cookies_db, "cookies")?;
    let conn = Connection::open(&cookies_db)?;
    ensure_moz_cookies_table(&conn)?;

    let now = now_micros();
    let mut added = 0usize;
    let mut skipped = 0usize;

    for cookie in cookies {
        let Some(value) = cookie_plain_value(cookie) else {
            skipped += 1;
            continue;
        };
        if cookie.name.is_empty() {
            skipped += 1;
            continue;
        }

        let host = normalize_host(&cookie.host_key);
        let expiry = chrome_expiry_to_firefox(cookie.expires_utc);
        let (same_site, raw_same_site) = map_same_site(cookie.same_site);

        if cookie_exists(&conn, &host, &cookie.name, &cookie.path)? {
            conn.execute(
                "UPDATE moz_cookies SET value = ?1, expiry = ?2, lastAccessed = ?3,
                 isSecure = ?4, isHttpOnly = ?5, sameSite = ?6, rawSameSite = ?7
                 WHERE host = ?8 AND name = ?9 AND path = ?10",
                params![
                    value,
                    expiry,
                    now,
                    cookie.is_secure as i64,
                    cookie.is_httponly as i64,
                    same_site,
                    raw_same_site,
                    host,
                    cookie.name,
                    cookie.path,
                ],
            )?;
        } else {
            conn.execute(
                "INSERT INTO moz_cookies (
                    originAttributes, name, value, host, path, expiry, lastAccessed,
                    creationTime, isSecure, isHttpOnly, inBrowserElement, sameSite,
                    rawSameSite, schemeMap
                 ) VALUES ('', ?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?8, 0, ?9, ?10, 2)",
                params![
                    cookie.name,
                    value,
                    host,
                    cookie.path,
                    expiry,
                    now,
                    cookie.is_secure as i64,
                    cookie.is_httponly as i64,
                    same_site,
                    raw_same_site,
                ],
            )?;
        }
        added += 1;
    }

    Ok(CookieWriteReport {
        cookies_added: added,
        cookies_skipped: skipped,
        backup_dir: Some(backup_dir),
    })
}

#[derive(Debug, Clone, Default)]
pub struct CookieWriteReport {
    pub cookies_added: usize,
    pub cookies_skipped: usize,
    pub backup_dir: Option<PathBuf>,
}

fn cookie_plain_value(cookie: &CookieRecord) -> Option<String> {
    match &cookie.value {
        SecretValue::PlainText(value) | SecretValue::Decrypted(value) => {
            if value.is_empty() {
                None
            } else {
                Some(value.clone())
            }
        }
        SecretValue::Encrypted(_) => None,
    }
}

fn normalize_host(host_key: &str) -> String {
    if host_key.is_empty() {
        return host_key.to_owned();
    }
    if host_key.starts_with('.') {
        host_key.to_owned()
    } else if host_key.starts_with("http://") || host_key.starts_with("https://") {
        host_key.to_owned()
    } else {
        format!(".{host_key}")
    }
}

fn chrome_expiry_to_firefox(chrome_utc: i64) -> i64 {
    if chrome_utc <= 0 {
        return 0;
    }
    const EPOCH_DIFF_SECS: i64 = 11_644_473_600;
    (chrome_utc / 1_000_000).saturating_sub(EPOCH_DIFF_SECS)
}

fn map_same_site(chrome: i64) -> (i64, i64) {
    match chrome {
        2 => (2, 2),
        1 => (1, 1),
        0 => (0, 0),
        _ => (0, 0),
    }
}

fn cookie_exists(conn: &Connection, host: &str, name: &str, path: &str) -> Result<bool, ImporterError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM moz_cookies WHERE host = ?1 AND name = ?2 AND path = ?3",
        params![host, name, path],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn ensure_moz_cookies_table(conn: &Connection) -> Result<(), ImporterError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS moz_cookies (
            id INTEGER PRIMARY KEY,
            originAttributes TEXT NOT NULL DEFAULT '',
            name TEXT,
            value TEXT,
            host TEXT,
            path TEXT,
            expiry INTEGER,
            lastAccessed INTEGER,
            creationTime INTEGER,
            isSecure INTEGER,
            isHttpOnly INTEGER,
            inBrowserElement INTEGER DEFAULT 0,
            sameSite INTEGER DEFAULT 0,
            rawSameSite INTEGER DEFAULT 0,
            schemeMap INTEGER DEFAULT 2
        );",
    )?;
    Ok(())
}

fn backup_file(db: &Path, label: &str) -> Result<PathBuf, ImporterError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup_dir = db
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("import-backups");
    fs::create_dir_all(&backup_dir)?;
    let dest = backup_dir.join(format!("{label}-{stamp}.sqlite"));
    fs::copy(db, &dest)?;
    Ok(backup_dir)
}

fn now_micros() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SecretValue;
    use std::fs;

    #[test]
    fn writes_decrypted_cookie() {
        let dir = std::env::temp_dir().join("aurexalis-gecko-cookies-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");

        let db = dir.join("cookies.sqlite");
        Connection::open(&db)
            .expect("open")
            .execute_batch(
                "CREATE TABLE moz_cookies (
                    id INTEGER PRIMARY KEY,
                    originAttributes TEXT NOT NULL DEFAULT '',
                    name TEXT, value TEXT, host TEXT, path TEXT,
                    expiry INTEGER, lastAccessed INTEGER, creationTime INTEGER,
                    isSecure INTEGER, isHttpOnly INTEGER,
                    inBrowserElement INTEGER DEFAULT 0,
                    sameSite INTEGER DEFAULT 0, rawSameSite INTEGER DEFAULT 0,
                    schemeMap INTEGER DEFAULT 2
                );",
            )
            .expect("schema");

        let report = write_cookies(
            &dir,
            &[CookieRecord {
                host_key: "example.com".to_owned(),
                name: "sid".to_owned(),
                path: "/".to_owned(),
                expires_utc: 0,
                is_secure: true,
                is_httponly: false,
                same_site: 1,
                value: SecretValue::Decrypted("abc".to_owned()),
            }],
        )
        .expect("write");

        assert_eq!(report.cookies_added, 1);
        assert_eq!(report.cookies_skipped, 0);

        let conn = Connection::open(&db).expect("open");
        let value: String = conn
            .query_row(
                "SELECT value FROM moz_cookies WHERE name = 'sid'",
                [],
                |r| r.get(0),
            )
            .expect("row");
        assert_eq!(value, "abc");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn skips_encrypted_without_plaintext() {
        let dir = std::env::temp_dir().join("aurexalis-gecko-cookies-skip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let db = dir.join("cookies.sqlite");
        Connection::open(&db)
            .expect("open")
            .execute_batch(
                "CREATE TABLE moz_cookies (
                    id INTEGER PRIMARY KEY,
                    originAttributes TEXT NOT NULL DEFAULT '',
                    name TEXT, value TEXT, host TEXT, path TEXT,
                    expiry INTEGER, lastAccessed INTEGER, creationTime INTEGER,
                    isSecure INTEGER, isHttpOnly INTEGER,
                    inBrowserElement INTEGER DEFAULT 0,
                    sameSite INTEGER DEFAULT 0, rawSameSite INTEGER DEFAULT 0,
                    schemeMap INTEGER DEFAULT 2
                );",
            )
            .expect("schema");

        let report = write_cookies(
            &dir,
            &[CookieRecord {
                host_key: "x.test".to_owned(),
                name: "secret".to_owned(),
                path: "/".to_owned(),
                expires_utc: 0,
                is_secure: false,
                is_httponly: false,
                same_site: -1,
                value: SecretValue::Encrypted(vec![1, 2, 3]),
            }],
        )
        .expect("write");

        assert_eq!(report.cookies_added, 0);
        assert_eq!(report.cookies_skipped, 1);
        let _ = fs::remove_dir_all(dir);
    }
}
