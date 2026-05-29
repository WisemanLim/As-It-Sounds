# src-tauri/src/core/db.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: SQLite 영속화 레이어. projects/media_files/segments/kv_settings 4개 테이블 CRUD.
- 진입점에서의 거리 / Distance from entrypoint: 3 (commands/pipeline → db)
- 라인 수 / LOC: 341
- 주요 export / Exports: `Db` (Clone), 모든 list/create/update/delete 메서드.
- 의존 모듈 / Depends on (internal): `super::error`, `super::models`
- 사용된 외부 패키지 / External packages used: `rusqlite`, `parking_lot::Mutex`, `uuid::Uuid`
- 부수효과 / Side effects: 디스크 파일 (`as-it-sounds.sqlite`) IO.
- 동시성 모델 / Concurrency model: 단일 `Connection` 을 `Arc<Mutex>` 로 wrap. 모든 호출이 직렬화.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=src-tauri/src/core/db.rs lines=1-14
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
```
**분석 / Analysis (Level 2)**
- `Db` 가 `Clone` 인 이유: `Arc<Mutex<...>>` 가 cheap clone → 여러 owner. AppState, Pipeline 양쪽이 같은 Connection 공유. / Cheap clone share.
- **No connection pooling**: 단일 connection 직렬화 모델. SQLite + WAL 에서는 multi-reader/single-writer 가능하므로 연결을 여러 개 두면 read 동시성 ↑. 현 구조는 모든 쿼리(read 포함)가 한 mutex 뒤에서 직렬. / Single-conn model — bottleneck for read concurrency.

**짚어볼 점 / Watch-outs**
- 큰 파일 list 등에서 짧은 락이라도 자주 잡힘 → IPC 명령 동시성에 영향 가능.

```rust path=src-tauri/src/core/db.rs lines=16-31
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
```
**분석 / Analysis (Level 2)**
- WAL 저널 모드 + foreign_keys ON → 일반적인 멀티프로세스 안전 + ON DELETE CASCADE 동작 보장. / WAL + FK enforcement, good defaults.
- `path: &PathBuf` 시그니처는 `&Path` 로 충분 (lint hint). / Clippy would suggest `&Path`.

```rust path=src-tauri/src/core/db.rs lines=33-73
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
```
**분석 / Analysis (Level 2)**
- `IF NOT EXISTS` 로 idempotent. 스키마 진화 시 version 컬럼/PRAGMA user_version 없으니 마이그레이션 시스템 부재. / No formal migration versioning.
- 두 개의 인덱스(`idx_media_files_project`, `idx_segments_file`) 는 list 쿼리에 정확히 매칭.
- `error TEXT` 컬럼은 nullable.

**짚어볼 점 / Watch-outs**
- 스키마 변경 (컬럼 추가/이름 변경) 시 사용자의 기존 DB 깨지면 대처 어려움. `PRAGMA user_version` + 절차적 migration 권장.

```rust path=src-tauri/src/core/db.rs lines=75-81
    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }
```
**분석 / Analysis (Level 2)**
- `SystemTime` 이 UNIX_EPOCH 이전이면 0. 평범한 환경에선 발생 안 함. / Pre-epoch returns 0 by default.
- `as i64` cast — 9999년쯤 overflow, 사실상 무한대. / Safe cast.

```rust path=src-tauri/src/core/db.rs lines=84-100
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
```
**분석 / Analysis (Level 2)**
- prepared statement → parameter binding 없음(여기는 파라미터 없는 쿼리). SQLi 무관. / No params, safe.
- `collect::<Result<Vec<_>, _>>()` 패턴: 첫 에러에서 fail-fast. / Idiomatic.

```rust path=src-tauri/src/core/db.rs lines=102-121
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
```
**분석 / Analysis (Level 2)**
- COUNT(*) + 1 로 기본 이름 생성. 동시 호출 시 race (둘 다 count=3 보고 "새 프로젝트 4" 두 번 생성). Single-user desktop 이라 거의 무방. / Race on name, harmless.
- `params!` 매크로로 prepared binding → SQLi 안전. / Parameterized, safe.

**짚어볼 점 / Watch-outs**
- count + 1 naming 이 사용자의 삭제/재생성으로 이름 중복 가능. UI 측에서 이름 수정 가능하므로 큰 문제 아님.

```rust path=src-tauri/src/core/db.rs lines=123-145
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
```
**분석 / Analysis (Level 2)**
- 모두 params 바인딩 → SQLi 무관. / Parameterized everywhere.
- UPDATE/DELETE 의 affected rows 검증 없음. 존재하지 않는 id 도 Ok 반환. / No "not found" signal.
- `touch_project` 는 pipeline 에서 호출되지 않음 → dead-ish, 후속 사용 가능성. / Possibly future use.

```rust path=src-tauri/src/core/db.rs lines=149-174
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
```
**분석 / Analysis (Level 2)**
- 인덱스 `idx_media_files_project` 활용. ORDER BY created_at ASC 는 인덱스 정렬은 아니지만 row 수 작아 부담 적음.
- `MediaKind::parse/FileStatus::parse` 의 silent fallback 으로 unknown 값이 default 로 매핑 (models.rs 참고).

```rust path=src-tauri/src/core/db.rs lines=176-211
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
```
**분석 / Analysis (Level 2)**
- 단일 transaction 안에서 file rows + project touch → 원자성 보장. / Atomic batch insert with project touch.
- `c.transaction()` 은 `let mut c` 가 필요한 이유. mutable borrow. / Hence `let mut c`.

```rust path=src-tauri/src/core/db.rs lines=213-244
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
```
**분석 / Analysis (Level 2)**
- `get_file` 와 `list_files` 가 동일 컬럼 매핑 코드 — 헬퍼로 추출 가능. / DRY opportunity.

```rust path=src-tauri/src/core/db.rs lines=246-267
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
```
**분석 / Analysis (Level 2)**
- pipeline 에서 자주 호출 → 매번 lock + execute. WAL + 단일 writer 모델이라 다른 reader 와 합쳐서 성능 OK.

```rust path=src-tauri/src/core/db.rs lines=269-318
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
```
**분석 / Analysis (Level 2)**
- `idx` 컬럼명 사용 (SQL 키워드 충돌 회피). 모델은 `index`. / `idx` in SQL, `index` in Rust.
- `INSERT OR REPLACE` → 동일 id 의 segment 가 들어오면 덮어씀. pipeline 마다 UUID 새로 생성되므로 사실상 INSERT.
- segment 단위 한 번씩 lock → 100 개 segment 시 100 번 lock 획득. 전체 파일을 한 transaction 으로 묶으면 throughput ↑. / Per-segment lock churn.

**짚어볼 점 / Watch-outs**
- segment 다량 삽입 시 트랜잭션 묶기로 큰 성능 개선 가능.

```rust path=src-tauri/src/core/db.rs lines=321-341
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
```
**분석 / Analysis (Level 2)**
- `ON CONFLICT DO UPDATE` 로 upsert. / SQLite upsert.
- 키-값 plain text. Settings 의 API key 가 여기로 저장됨 (평문).

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 모든 쿼리 parameterized → SQLi 안전.
  2. WAL + FK + ON DELETE CASCADE 로 데이터 무결성 ↑.
  3. `add_files` 가 transaction 으로 원자성 보장.
- 위험 / Risks:
  1. 단일 connection mutex 직렬화 → multi-reader 이점 못 살림.
  2. 마이그레이션 버전 관리 없음 → 스키마 진화 어려움.
  3. segment 삽입을 트랜잭션으로 묶지 않아 잠금 경합 ↑.
- 개선 권고 / Recommendations:
  1. `r2d2_sqlite` 등 connection pool, read/write 분리.
  2. `PRAGMA user_version` + 절차적 migration 도입.
  3. `insert_segments_bulk(&[Segment])` 트랜잭션 메서드.
- 재읽기 권고 레벨 / Suggested re-read level: 3 — 성능/스키마 진화/멀티사용자 (web-server) 시 권장.
