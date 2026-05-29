# `web-server/src/` 디렉터리 요약

> [← INDEX.md](../../INDEX.md)

## 파일 목록

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| main.rs | 376 | [main.rs.md](./main.rs.md) |
| pipeline.rs | 155 | [pipeline.rs.md](./pipeline.rs.md) |
| core.rs | 30 | [core.rs.md](./core.rs.md) |

## 구조 요약

axum 기반 HTTP/SSE 서버. `core.rs`가 `#[path]`로 desktop core를 그대로 흡수, `pipeline.rs`가 broadcast 채널로 PipelineEvent를 SSE에 다중 송출, `main.rs`가 라우터+핸들러+멀티파트 업로드+ServeFile 정적 응답을 처리.

## 핵심 의존

- `axum`, `tower-http` (CORS/trace/ServeFile/limit)
- `tokio::sync::broadcast`
- 내부: `crate::core::*` (재익스포트된 desktop core)

## 리스크 시그널

- `CorsLayer::new().allow_origin(Any)` — 모든 origin 허용. Dockerfile.web 의도가 백엔드만일지라도 위험.
- 멀티파트 업로드가 chunk마다 fs write — 디스크 용량/공간 예외 미처리.
- `files_remove`가 fs::remove_file 결과 무시 — orphan file 가능.
- `events_sse` BroadcastStream lag 시 메시지 손실 가능 — 새 클라이언트는 채널 사이즈(1024) 이전 이벤트 못 받음.
