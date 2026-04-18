use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::{params, Connection};
use uuid::Uuid;

use super::error::AppResult;
use super::models::{FileStatus, MediaFile, MediaKind, Project, Segment};

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(path: &PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;",
        )?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS media_files (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                kind TEXT NOT NULL,
                size_bytes INTEGER,
                duration_ms INTEGER,
                status TEXT NOT NULL DEFAULT 'pending',
                error TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_media_files_project ON media_files(project_id);
            CREATE TABLE IF NOT EXISTS segments (
                id TEXT PRIMARY KEY,
                file_id TEXT NOT NULL REFERENCES media_files(id) ON DELETE CASCADE,
                idx INTEGER NOT NULL,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER NOT NULL,
                text_en TEXT NOT NULL,
                text_hangul TEXT,
                text_ko TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_segments_file ON segments(file_id, idx);
            CREATE TABLE IF NOT EXISTS kv_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    // projects -----------------------------------------------------------

    pub fn list_projects(&self) -> AppResult<Vec<Project>> {
        let c = self.conn.lock();
        let mut stmt = c.prepare(
            "SELECT id, name, created_at, updated_at FROM projects ORDER BY updated_at DESC",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(Project {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    created_at: r.get(2)?,
                    updated_at: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn create_project(&self, name: Option<String>) -> AppResult<Project> {
        let id = Uuid::new_v4().to_string();
        let now = Self::now_ms();
        let c = self.conn.lock();
        let count: i64 =
            c.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))?;
        let name = name.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| {
            format!("새 프로젝트 {}", count + 1)
        });
        c.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            params![id, name, now],
        )?;
        Ok(Project {
            id,
            name,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn rename_project(&self, id: &str, name: &str) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, Self::now_ms(), id],
        )?;
        Ok(())
    }

    pub fn touch_project(&self, id: &str) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute(
            "UPDATE projects SET updated_at = ?1 WHERE id = ?2",
            params![Self::now_ms(), id],
        )?;
        Ok(())
    }

    pub fn delete_project(&self, id: &str) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    // files --------------------------------------------------------------

    pub fn list_files(&self, project_id: &str) -> AppResult<Vec<MediaFile>> {
        let c = self.conn.lock();
        let mut stmt = c.prepare(
            "SELECT id, project_id, name, path, kind, size_bytes, duration_ms, status, error, created_at
             FROM media_files WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt
            .query_map(params![project_id], |r| {
                let kind: String = r.get(4)?;
                let status: String = r.get(7)?;
                Ok(MediaFile {
                    id: r.get(0)?,
                    project_id: r.get(1)?,
                    name: r.get(2)?,
                    path: r.get(3)?,
                    kind: MediaKind::parse(&kind),
                    size_bytes: r.get(5)?,
                    duration_ms: r.get(6)?,
                    status: FileStatus::parse(&status),
                    error: r.get(8)?,
                    created_at: r.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn add_files(
        &self,
        project_id: &str,
        inputs: &[(String, String, MediaKind, Option<i64>)],
    ) -> AppResult<Vec<MediaFile>> {
        let now = Self::now_ms();
        let mut created = Vec::new();
        let mut c = self.conn.lock();
        let tx = c.transaction()?;
        for (name, path, kind, size) in inputs {
            let id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO media_files (id, project_id, name, path, kind, size_bytes, duration_ms, status, error, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, 'pending', NULL, ?7)",
                params![id, project_id, name, path, kind.as_str(), size, now],
            )?;
            created.push(MediaFile {
                id,
                project_id: project_id.to_string(),
                name: name.clone(),
                path: path.clone(),
                kind: *kind,
                size_bytes: *size,
                duration_ms: None,
                status: FileStatus::Pending,
                error: None,
                created_at: now,
            });
        }
        tx.execute(
            "UPDATE projects SET updated_at = ?1 WHERE id = ?2",
            params![now, project_id],
        )?;
        tx.commit()?;
        Ok(created)
    }

    pub fn get_file(&self, id: &str) -> AppResult<Option<MediaFile>> {
        let c = self.conn.lock();
        let mut stmt = c.prepare(
            "SELECT id, project_id, name, path, kind, size_bytes, duration_ms, status, error, created_at
             FROM media_files WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(r) = rows.next()? {
            let kind: String = r.get(4)?;
            let status: String = r.get(7)?;
            Ok(Some(MediaFile {
                id: r.get(0)?,
                project_id: r.get(1)?,
                name: r.get(2)?,
                path: r.get(3)?,
                kind: MediaKind::parse(&kind),
                size_bytes: r.get(5)?,
                duration_ms: r.get(6)?,
                status: FileStatus::parse(&status),
                error: r.get(8)?,
                created_at: r.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn remove_file(&self, id: &str) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute("DELETE FROM media_files WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_file_status(
        &self,
        id: &str,
        status: FileStatus,
        error: Option<&str>,
    ) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute(
            "UPDATE media_files SET status = ?1, error = ?2 WHERE id = ?3",
            params![status.as_str(), error, id],
        )?;
        Ok(())
    }

    pub fn set_file_duration(&self, id: &str, duration_ms: i64) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute(
            "UPDATE media_files SET duration_ms = ?1 WHERE id = ?2",
            params![duration_ms, id],
        )?;
        Ok(())
    }

    // segments -----------------------------------------------------------

    pub fn list_segments(&self, file_id: &str) -> AppResult<Vec<Segment>> {
        let c = self.conn.lock();
        let mut stmt = c.prepare(
            "SELECT id, file_id, idx, start_ms, end_ms, text_en, text_hangul, text_ko
             FROM segments WHERE file_id = ?1 ORDER BY idx ASC",
        )?;
        let rows = stmt
            .query_map(params![file_id], |r| {
                Ok(Segment {
                    id: r.get(0)?,
                    file_id: r.get(1)?,
                    index: r.get(2)?,
                    start_ms: r.get(3)?,
                    end_ms: r.get(4)?,
                    text_en: r.get(5)?,
                    text_hangul: r.get(6)?,
                    text_ko: r.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn clear_segments(&self, file_id: &str) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute("DELETE FROM segments WHERE file_id = ?1", params![file_id])?;
        Ok(())
    }

    pub fn insert_segment(&self, seg: &Segment) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute(
            "INSERT OR REPLACE INTO segments (id, file_id, idx, start_ms, end_ms, text_en, text_hangul, text_ko)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                seg.id,
                seg.file_id,
                seg.index,
                seg.start_ms,
                seg.end_ms,
                seg.text_en,
                seg.text_hangul,
                seg.text_ko,
            ],
        )?;
        Ok(())
    }

    // settings -----------------------------------------------------------

    pub fn get_setting(&self, key: &str) -> AppResult<Option<String>> {
        let c = self.conn.lock();
        let mut stmt = c.prepare("SELECT value FROM kv_settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(r) = rows.next()? {
            Ok(Some(r.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        let c = self.conn.lock();
        c.execute(
            "INSERT INTO kv_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}
