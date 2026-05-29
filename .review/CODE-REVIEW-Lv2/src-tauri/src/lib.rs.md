# src-tauri/src/lib.rs — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Tauri 데스크톱 앱 엔트리 (`run()`). 플러그인 등록, `AppState` 초기화(Db, AsrEngine, Pipeline), invoke_handler 라우팅. / Tauri desktop app entry. Plugin registration, app state init, IPC command routing.
- 진입점에서의 거리 / Distance from entrypoint: 1 (main.rs → lib.rs)
- 라인 수 / LOC: 52
- 주요 export / Exports: `pub fn run()`, `pub mod core`
- 의존 모듈 / Depends on (internal): `commands`, `core::asr`, `core::db`, `core::pipeline`
- 사용된 외부 패키지 / External packages used: `tauri`, `tauri_plugin_opener`, `tauri_plugin_dialog`, `tauri_plugin_fs`
- 부수효과 / Side effects: 디스크 디렉토리 생성, SQLite 파일 open, Whisper 모델 디렉토리 준비, OS 윈도우 생성.
- 동시성 모델 / Concurrency model: Tauri 내부 tokio 런타임. `Arc<AsrEngine>`, `Arc<Pipeline>` 공유.
- 테스트 커버리지 흔적 / Test coverage signal: 없음 (no `#[test]`).

```rust path=src-tauri/src/lib.rs lines=1-12
mod commands;
pub mod core;

use std::sync::Arc;

use tauri::Manager;

use crate::commands::AppState;
use crate::core::asr::AsrEngine;
use crate::core::db::Db;
use crate::core::pipeline::Pipeline;
```
**분석 / Analysis (Level 2)**
- `mod commands` 는 private(내부 전용), `pub mod core` 는 web-server에서 `#[path]` 재참조하기 때문에 pub 노출 필요. / `commands` is internal, `core` is `pub` because web-server re-imports it via `#[path]`.
- `Arc` import 가 있지만 본 함수에서 `Arc::new(...)` 만 쓰임. 모듈 경계가 명시적이라 가독성 ↑. / `Arc` only used inside setup closure.

```rust path=src-tauri/src/lib.rs lines=13-18
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
```
**분석 / Analysis (Level 2)**
- `#[cfg_attr(mobile, tauri::mobile_entry_point)]` 는 iOS/Android 빌드 시 진입점을 `mobile_entry_point` 매크로로 감싼다. 데스크톱 빌드에는 영향 없음. / Mobile target wraps entry; no-op on desktop.
- 세 개의 플러그인(opener, dialog, fs)은 프론트엔드에서 파일 선택과 외부 URL 열기에 사용. **iOS 샌드박스**에서는 dialog/fs 의 동작 범위가 제한됨. / FS/dialog plugins are sandbox-limited on iOS.

**짚어볼 점 / Watch-outs**
- 플러그인 초기화 실패는 패닉 없이 무시되는데, Tauri builder 가 내부적으로 어떻게 처리하는지 확인 필요.

```rust path=src-tauri/src/lib.rs lines=19-34
        .setup(|app| {
            let handle = app.handle();
            let data_dir = handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("as-it-sounds"));
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("as-it-sounds.sqlite");
            let db = Db::open(&db_path).expect("open db");
            let models_dir = data_dir.join("models");
            let asr = Arc::new(AsrEngine::new(models_dir).expect("init asr"));
            let pipeline =
                Arc::new(Pipeline::new(db.clone(), asr.clone()).expect("init pipeline"));
            app.manage(AppState { db, pipeline });
            Ok(())
        })
```
**분석 / Analysis (Level 2)**
- `app_data_dir()` 실패 시 `temp_dir/as-it-sounds` 폴백. macOS/Windows에서는 보통 성공. / Fallback to temp dir is reasonable.
- `create_dir_all(&data_dir).ok()` 는 에러를 의도적으로 무시. 만약 부모 디렉토리 생성 실패면 다음 줄의 `Db::open` 도 실패하여 `expect("open db")` 가 패닉. / Silently swallowed; subsequent `expect` will panic.
- `expect("open db")`, `expect("init asr")`, `expect("init pipeline")` 세 곳 모두 부팅 시점 패닉. 사용자에게는 윈도우가 열리기 전에 크래시로 보임. / Three panic points at boot.
- `Db::open` 은 내부에서 `migrate()` 호출하므로 마이그레이션 실패도 여기서 패닉.

**짚어볼 점 / Watch-outs**
- (CWE-755) 설정 단계 패닉 → UI 로그 없이 종료. 차후 사용자에게 보일 친절한 오류 다이얼로그 필요.
- `db.clone()` 는 `Arc<Mutex<Connection>>` 의 cheap clone (Db 가 `#[derive(Clone)]`).

```rust path=src-tauri/src/lib.rs lines=35-52
        .invoke_handler(tauri::generate_handler![
            commands::project_list,
            commands::project_create,
            commands::project_rename,
            commands::project_delete,
            commands::file_list,
            commands::file_add_many,
            commands::file_remove,
            commands::file_reset,
            commands::segments_list,
            commands::settings_get,
            commands::settings_set,
            commands::pipeline_run,
            commands::translit_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```
**분석 / Analysis (Level 2)**
- `generate_handler!` 매크로가 컴파일 타임에 IPC 라우터를 생성. 모든 command 가 `#[tauri::command]` 어트리뷰트를 가져야 함. / Compile-time IPC dispatch table.
- `convert_file_src_helper` 가 `commands.rs` 에 정의되어 있지만 여기 등록되지 않음 → dead code. / `convert_file_src_helper` defined but not registered — dead.
- 최종 `.expect(...)` 는 윈도우 표시 단계 실패에도 패닉.

**짚어볼 점 / Watch-outs**
- 사용자가 IPC 명령을 추가할 때마다 두 곳(commands.rs + 이 매크로) 수정 필요 → 빠뜨리기 쉬움. 새 명령이 호출 안 되면 invoke_handler 등록을 우선 확인.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  1. 부팅 흐름이 한 함수에 모두 보여 흐름 파악이 빠름.
  2. `Arc<...>` 와 `manage(AppState{..})` 패턴이 Tauri 권장 방식 그대로.
  3. 모바일 cfg_attr 분기가 명시적.
- 위험 / Risks:
  1. 3중 `expect()` 부팅 패닉 → 사용자 친화 오류 없음 (CWE-755).
  2. `convert_file_src_helper` 등록 누락으로 dead code 생성.
  3. `create_dir_all(&data_dir).ok()` 에러 스왈로잉.
- 개선 권고 / Recommendations:
  1. `setup` 클로저에서 `Result<_, Box<dyn Error>>` 활용해 패닉 대신 ‘초기화 실패' 로그 + 다이얼로그.
  2. invoke_handler 미등록 핸들러를 컴파일 타임에 잡는 매크로/테스트.
  3. `iOS` 빌드를 가정하면 ffmpeg 의존성을 `cfg!(target_os)` 로 분기하여 가시화.
- 재읽기 권고 레벨 / Suggested re-read level: 3 — Tauri builder/manage 라이프사이클을 더 깊이 보려면 권장.
