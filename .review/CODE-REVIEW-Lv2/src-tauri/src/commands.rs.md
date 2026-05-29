# src-tauri/src/commands.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Tauri IPC 명령(`#[tauri::command]`) 어댑터. 프론트엔드에서 호출 가능한 Rust 함수 모음.
- 진입점에서의 거리 / Distance from entrypoint: 2 (main → lib::run → 여기로 등록)
- 라인 수 / LOC: 144
- 주요 export / Exports: `AppState`, `project_list/create/rename/delete`, `file_*`, `segments_list`, `settings_*`, `pipeline_run`, `translit_preview`, `convert_file_src_helper`
- 의존 모듈 / Depends on (internal): `core::db`, `core::pipeline`, `core::settings`, `core::translit`, `core::models`, `core::error`
- 사용된 외부 패키지 / External packages used: `tauri` (State, AppHandle, Manager), `std::sync::Arc`
- 부수효과 / Side effects: DB 쓰기/읽기, 파이프라인 비동기 실행 트리거.
- 동시성 모델 / Concurrency model: 각 command 는 Tauri 가 별도 task 로 처리. 내부에서 `Db` 의 `Mutex` 가 직렬화.
- 테스트 커버리지 흔적 / Test coverage signal: 없음.

```rust path=src-tauri/src/commands.rs lines=1-17
use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::core::db::Db;
use crate::core::error::AppResult;
use crate::core::models::{
    FileStatus, MediaFile, MediaKind, NewFileInput, Project, Segment, Settings,
};
use crate::core::pipeline::Pipeline;
use crate::core::settings as settings_store;
use crate::core::translit;

pub struct AppState {
    pub db: Db,
    pub pipeline: Arc<Pipeline>,
}
```
**분석 / Analysis (Level 2)**
- `AppState` 는 Tauri `manage()` 에 저장돼 모든 command 가 `State<'_, AppState>` 로 접근. / Global app state.
- `Manager` import 는 여기서는 사용되지 않음 (lib.rs 에서 사용). 컴파일러는 warning 만 낼 수 있음. / `Manager` is unused in this file.

**짚어볼 점 / Watch-outs**
- `Db` 가 `Clone` 이라 `pipeline` 안에 또 한 번 보관됨 → AppState 와 Pipeline 의 Db 가 같은 Arc<Mutex<Connection>> 를 공유. OK.

```rust path=src-tauri/src/commands.rs lines=19-30
#[tauri::command]
pub async fn project_list(state: State<'_, AppState>) -> AppResult<Vec<Project>> {
    state.db.list_projects()
}

#[tauri::command]
pub async fn project_create(
    state: State<'_, AppState>,
    name: Option<String>,
) -> AppResult<Project> {
    state.db.create_project(name)
}
```
**분석 / Analysis (Level 2)**
- 함수 시그니처는 `async` 이지만 본문은 **동기 호출**(SQLite mutex). Tauri runtime 이 자체 thread pool 에서 실행하므로 OK이지만, 다른 async 작업과 같이 돌아갈 때 mutex 보유 중 await 점이 없어 데드락 위험 낮음. / Async signature wrapping sync DB call.
- `AppResult` 는 `Serialize` 구현이 있어 프론트로 에러 문자열 직렬화됨.

**짚어볼 점 / Watch-outs**
- 동기 DB 호출이 UI thread 를 막지는 않지만 long-running 시 다른 command 와 mutex 경합.

```rust path=src-tauri/src/commands.rs lines=32-47
#[tauri::command]
pub async fn project_rename(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> AppResult<()> {
    state.db.rename_project(&id, &name)
}

#[tauri::command]
pub async fn project_delete(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    state.db.delete_project(&id)
}
```
**분석 / Analysis (Level 2)**
- `id` 와 `name` 모두 사용자(=프론트엔드) 입력. DB 는 prepared statement 사용하므로 SQL injection 무력화. / Parameterized — SQLi safe.

**짚어볼 점 / Watch-outs**
- 프로젝트 rename 시 이름 길이/문자 검증 없음. UI 단에서만 신뢰 → 비ASCII, 매우 긴 이름 들어가도 그대로 저장.

```rust path=src-tauri/src/commands.rs lines=49-78
#[tauri::command]
pub async fn file_list(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<MediaFile>> {
    state.db.list_files(&project_id)
}

#[tauri::command]
pub async fn file_add_many(
    state: State<'_, AppState>,
    project_id: String,
    inputs: Vec<NewFileInput>,
) -> AppResult<Vec<MediaFile>> {
    let prepared: Vec<_> = inputs
        .into_iter()
        .map(|i| {
            let kind = i
                .kind
                .as_deref()
                .map(|k| match k {
                    "video" => MediaKind::Video,
                    _ => MediaKind::Audio,
                })
                .unwrap_or_else(|| MediaKind::from_name(&i.name));
            (i.name, i.path, kind, i.size_bytes)
        })
        .collect();
    state.db.add_files(&project_id, &prepared)
}
```
**분석 / Analysis (Level 2)**
- `kind` 가 명시되지 않으면 파일명 확장자로 `MediaKind::from_name` 결정. 보통 안전. / Default kind by extension.
- `i.path` 는 프론트가 보낸 임의 경로. 파이프라인 단계에서 `PathBuf::from` 후 `exists()` 검증하므로 실행 시점 missing 파일 처리 OK. 다만 **다른 프로젝트의 파일 경로**로 row 를 만드는 것을 막지 않음 (logical issue, not security).

**짚어볼 점 / Watch-outs**
- (CWE-22) 경로 자체 검증 없음. 같은 머신 사용자라 가정하지만 web-server 빌드에선 다른 얘기 (그쪽은 multipart 업로드로 격리).

```rust path=src-tauri/src/commands.rs lines=80-104
#[tauri::command]
pub async fn file_remove(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    state.db.remove_file(&id)
}

#[tauri::command]
pub async fn file_reset(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    state.db.clear_segments(&id)?;
    state.db.set_file_status(&id, FileStatus::Pending, None)?;
    Ok(())
}

#[tauri::command]
pub async fn segments_list(
    state: State<'_, AppState>,
    file_id: String,
) -> AppResult<Vec<Segment>> {
    state.db.list_segments(&file_id)
}
```
**분석 / Analysis (Level 2)**
- `file_remove` 는 원본 파일을 디스크에서 지우지 않음 (DB row 만). web-server 쪽 `files_remove` 는 disk 파일도 정리. 데스크톱은 사용자가 직접 추가한 경로라서 의도적인 것일 수 있음. / Desktop intentionally keeps source file.
- `file_reset` 두 개의 DB call 사이에 트랜잭션 없음. 첫 호출 후 패닉이면 segments 만 비고 status 는 그대로. 실제 영향 적음(다시 reset 가능).

**짚어볼 점 / Watch-outs**
- 트랜잭션화 또는 db 메서드에 `reset_file` 단일 호출로 합쳐 원자성 보장 검토.

```rust path=src-tauri/src/commands.rs lines=106-118
#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> AppResult<Settings> {
    settings_store::load(&state.db)
}

#[tauri::command]
pub async fn settings_set(
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<Settings> {
    settings_store::save(&state.db, &settings)?;
    Ok(settings)
}
```
**분석 / Analysis (Level 2)**
- 저장 후 입력 그대로 반환. 직렬화 round-trip 검증은 안함. / Echoes back input after save.

**짚어볼 점 / Watch-outs**
- (Secret handling) `Settings.translate_api_key` 는 평문으로 SQLite kv 에 JSON 저장. 디스크 평문 노출 → README 에서 명시 권장. (CWE-312)

```rust path=src-tauri/src/commands.rs lines=120-132
#[tauri::command]
pub async fn pipeline_run(
    app: AppHandle,
    state: State<'_, AppState>,
    file_ids: Vec<String>,
) -> AppResult<()> {
    let settings = settings_store::load(&state.db)?;
    let pipeline = state.pipeline.clone();
    tauri::async_runtime::spawn(async move {
        let _ = pipeline.run(app, file_ids, settings).await;
    });
    Ok(())
}
```
**분석 / Analysis (Level 2)**
- 비동기 spawn → command 는 즉시 반환, 결과는 event 로 emit. / Fire-and-forget.
- `let _ = ...` 로 에러 swallow (CWE-252). `Pipeline::run` 은 내부적으로 per-file 에러를 emit 하므로 외부 결과를 무시해도 큰 정보 손실 없음. 다만 ffmpeg/모델 부재 같은 사전 단계 에러는 사용자에게 알릴 길이 없을 수 있음.

**짚어볼 점 / Watch-outs**
- 동시에 여러 번 `pipeline_run` 호출 시 spawn 이 누적. Whisper context mutex 가 직렬화하지만 ffmpeg 는 병렬 실행 가능 → CPU 과부하 위험.
- spawn 된 future 의 panic 은 task drop 으로 사라짐. 로깅 권장.

```rust path=src-tauri/src/commands.rs lines=134-144
#[tauri::command]
pub fn translit_preview(text: String) -> String {
    translit::transliterate(&text)
}

#[tauri::command]
pub fn convert_file_src_helper(app: AppHandle, path: String) -> AppResult<String> {
    // Returns a tauri asset URL we can serve from the frontend.
    let _ = app; // reserved for future use
    Ok(path)
}
```
**분석 / Analysis (Level 2)**
- `translit_preview` 는 동기 command. translit 은 순수 CPU + 메모리이지만 매우 긴 입력에 대해 호출 thread 가 잠시 동안 점유될 수 있음. / Sync; long input could block briefly.
- `convert_file_src_helper` 는 입력을 그대로 돌려줌 → 사실상 noop. invoke_handler 에 등록되지도 않아 호출되지도 않음. **Dead code**.

**짚어볼 점 / Watch-outs**
- `convert_file_src_helper` 는 제거하거나 실제 `tauri::path::asset_url` 호출로 채우는 결정 필요.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 각 command 가 한 줄~다섯 줄로 간결, IPC 어댑터 역할에 충실.
  2. SQL 은 모두 prepared statement 위에서 동작하여 SQLi 노출 없음.
  3. 에러 타입 `AppResult` 의 `Serialize` 통해 프론트로 일관된 에러 전달.
- 위험 / Risks:
  1. (CWE-252) `pipeline_run` 의 `let _ = ...` 에러 swallow.
  2. `convert_file_src_helper` 미사용 dead code.
  3. `file_reset` 2-step DB 변경의 원자성 부재.
- 개선 권고 / Recommendations:
  1. spawn 결과를 `tracing::warn!` 로 로깅, 또는 emit event 로 통보.
  2. dead code 제거 또는 register.
  3. `db.reset_file(id)` 메서드로 트랜잭션 묶기.
- 재읽기 권고 레벨 / Suggested re-read level: 2 — 현재 레벨로 충분.
