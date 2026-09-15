use std::path::Path;

use rusqlite::Connection;

pub fn read_sqlite_value(db_path: &Path, key: &str) -> Option<String> {
    if !db_path.exists() {
        return None;
    }

    let conn = open_read_only(db_path)?;

    let value: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?1 LIMIT 1",
            [key],
            |row| row.get(0),
        )
        .ok()?;

    (!value.is_empty()).then_some(value)
}

fn open_read_only(db_path: &Path) -> Option<Connection> {
    let path_str = db_path.to_string_lossy();
    let uri_ro = format!("file:{}?mode=ro", path_str.replace('\\', "/"));
    if let Ok(conn) = Connection::open_with_flags(
        &uri_ro,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    ) {
        return Some(conn);
    }

    let uri_immutable = format!("file:{}?mode=ro&immutable=1", path_str.replace('\\', "/"));
    Connection::open_with_flags(
        &uri_immutable,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .ok()
    .or_else(|| {
        Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).ok()
    })
}
