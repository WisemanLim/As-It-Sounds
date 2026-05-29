# web-server/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| src/main.rs | 376 | Axum HTTP/SSE 서버 + 라우트 / Axum server + routes | [src/main.rs.md](src/main.rs.md) |
| src/core.rs | 30 | `#[path]` 으로 src-tauri/src/core 재사용 / Reuse desktop core | [src/core.rs.md](src/core.rs.md) |
| src/pipeline.rs | 155 | broadcast 채널 기반 web pipeline / Web pipeline | [src/pipeline.rs.md](src/pipeline.rs.md) |

## 디렉토리 구조 / Structure Summary
Tauri 명령과 동등한 HTTP API 를 Axum 으로 노출하는 두 번째 크레이트. `src/core.rs` 는 src-tauri 의 core 모듈들을 `#[path]` 어트리뷰트로 빌려와 코드 중복 없이 ASR/ffmpeg/translit/DB/settings 를 재사용한다. `src/pipeline.rs` 는 데스크톱 pipeline 의 대응본으로 `AppHandle::emit` 대신 `tokio::sync::broadcast` 채널로 PipelineEvent 를 발행하며, 이 채널은 `main.rs::events_sse` 핸들러가 SSE 로 클라이언트에 중계한다. 파일 업로드는 `/api/projects/:id/files` 의 multipart 핸들러가 받아 `uploads_dir/<UUID>.<ext>` 로 저장.

## 주요 의존 관계 / Key Dependencies
- 외부: `axum`, `tower`, `tower-http`, `tokio`, `tokio-stream`, `futures-util`, `mime_guess`, `tracing`, `serde`/`serde_json`.
- 내부: `core` (=src-tauri 코드 재사용), `pipeline` (자체 정의).
- 환경 변수: `AIS_DATA_DIR` (기본 `/data`), `AIS_MAX_UPLOAD_MB` (기본 2048), `AIS_BIND` (기본 `0.0.0.0:8080`).

## 위험 신호 / Risk Signals
- (CWE-306) **인증/인가 완전 부재** — 사설 네트워크/localhost 전용 가정 필요.
- (CWE-942) `CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)` — 전면 개방.
- (CWE-209) 모든 에러를 500 + 원본 메시지로 응답 — 정보 누출.
- Code duplication: `pipeline.rs` 는 src-tauri 와 거의 동일 구조.
