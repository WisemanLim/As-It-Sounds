# web-server/src/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| main.rs | 376 | Axum 라우트/핸들러/SSE/multipart 업로드 / Routes & SSE | [main.rs.md](main.rs.md) |
| core.rs | 30 | desktop core 모듈을 `#[path]` 로 mount / Reuse via #[path] | [core.rs.md](core.rs.md) |
| pipeline.rs | 155 | broadcast 기반 pipeline / Broadcast pipeline | [pipeline.rs.md](pipeline.rs.md) |

## 디렉토리 구조 / Structure Summary
3개 파일짜리 단순 구조. `main.rs` 가 `#[tokio::main]` 진입점이며 라우터 트리, AppState, 11개 핸들러(healthz, projects, files, segments, settings, pipeline_run, translit_preview, events_sse) 가 모두 한 파일에 있다. `core.rs` 는 `#[path = "../../src-tauri/src/core/..."]` 8 줄로 desktop core 를 통째 mount. `pipeline.rs` 는 broadcast Sender 를 들고 desktop pipeline 과 동일 ETL 흐름을 수행.

## 주요 의존 관계 / Key Dependencies
- `main` → `core::{db, asr, settings, translit, models, error}` + `pipeline::Pipeline`.
- `pipeline` → `core::{asr, db, media, translate, translit, models, error}`.
- SSE: `main::events_sse` ← `pipeline.events` (broadcast::Sender) ← `pipeline.emit(...)`.

## 위험 신호 / Risk Signals
- 모든 보안 정책(authn/authz/CORS) 이 한 파일 (`main.rs`) 에 집중 — 변경 시 단일 지점이지만 동시에 보안 한계가 노출된 위치.
- `pipeline.rs` 가 desktop 과 거의 동일 → 한쪽 fix 가 다른쪽에 자동 반영되지 않음 (drift).
