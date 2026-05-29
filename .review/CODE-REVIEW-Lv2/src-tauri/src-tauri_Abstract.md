# src-tauri/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| src/main.rs | 6 | 프로세스 엔트리, lib::run 호출 / Process entry | [main.rs.md](src/main.rs.md) |
| src/lib.rs | 52 | Tauri 빌더 + AppState 초기화 / Tauri builder + state | [lib.rs.md](src/lib.rs.md) |
| src/commands.rs | 144 | IPC 명령 어댑터 / IPC command adapters | [commands.rs.md](src/commands.rs.md) |
| src/core/mod.rs | 11 | core 하위 모듈 선언 / core module declarations | [core/mod.rs.md](src/core/mod.rs.md) |
| src/core/error.rs | 58 | AppError/AppResult / Error types | [core/error.rs.md](src/core/error.rs.md) |
| src/core/models.rs | 154 | 도메인 모델 + serde / Domain models + serde | [core/models.rs.md](src/core/models.rs.md) |
| src/core/db.rs | 341 | SQLite CRUD / SQLite layer | [core/db.rs.md](src/core/db.rs.md) |
| src/core/asr.rs | 187 | Whisper engine wrapper / Whisper wrapper | [core/asr.rs.md](src/core/asr.rs.md) |
| src/core/media.rs | 131 | ffmpeg PCM decode / ffmpeg decode | [core/media.rs.md](src/core/media.rs.md) |
| src/core/translate.rs | 110 | EN→KO 번역 어댑터 / Translation adapter | [core/translate.rs.md](src/core/translate.rs.md) |
| src/core/translit.rs | 1713 | 다국어 음차 → 한글 / Multilingual phonetic→Hangul | [core/translit.rs.md](src/core/translit.rs.md) |
| src/core/pipeline.rs | 179 | ETL 오케스트레이터 (Tauri emit) / Pipeline orchestrator | [core/pipeline.rs.md](src/core/pipeline.rs.md) |
| src/core/settings.rs | 21 | settings JSON kv wrapper | [core/settings.rs.md](src/core/settings.rs.md) |

## 디렉토리 구조 / Structure Summary
`src-tauri/` 는 Tauri 데스크톱 앱의 Rust 백엔드 크레이트다. `main.rs` → `lib.rs::run()` → `commands.rs` 의 `#[tauri::command]` 핸들러 → `core/*` 도메인 로직 → `core/pipeline.rs` 오케스트레이터가 ffmpeg/Whisper/translate/translit 을 호출하여 `core/db.rs` 의 SQLite 에 결과를 영속화하고 Tauri `AppHandle::emit` 으로 프론트에 이벤트를 보낸다. `core/*` 모듈들은 `web-server/` 가 `#[path]` 어트리뷰트로 직접 재사용하므로 (pipeline 제외) **단일 진리원천(single source of truth)** 역할을 한다.

## 주요 의존 관계 / Key Dependencies
- 외부: `tauri`, `tauri-plugin-{opener,dialog,fs}`, `whisper-rs`, `rusqlite`, `reqwest`, `tokio`, `parking_lot`, `once_cell`, `thiserror`, `serde`, `uuid`.
- 내부: `commands` → `core::*`. `core::pipeline` → `core::{asr, media, translate, translit, db, models}`.
- 빌드: `cargo build` 시 Tauri 가 frontend 빌드(dist/) 를 정적으로 임베드.

## 위험 신호 / Risk Signals
- (CWE-755) lib.rs setup 클로저의 `expect("open db")` 등 3중 panic — 부팅 실패 시 사용자에게 어떤 다이얼로그도 안 보임.
- (CWE-770/iOS) `core/media.rs` 는 외부 `ffmpeg` 바이너리에 spawn 의존 — iOS 샌드박스에서는 빌드/실행 불가. iOS 지원 시 libav 정적 링크 또는 AVFoundation 대체 필요.
- (Concurrency) `core/asr.rs` 가 추론 전 구간을 `parking_lot::Mutex` 로 직렬화 — 동시 transcribe 호출의 fairness 문제 가능.
