# ipc/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| tauriClient.ts | 54 | Tauri invoke + listen 어댑터 | [tauriClient.ts.md](./tauriClient.ts.md) |
| webClient.ts | 121 | fetch + EventSource 어댑터 (web 모드) | [webClient.ts.md](./webClient.ts.md) |
| client.ts | 16 | surface dispatcher (isTauri 분기) | [client.ts.md](./client.ts.md) |

## 디렉토리 구조 / Structure Summary
desktop / web 두 surface 의 통신 어댑터를 한 폴더에 배치하고 `client.ts` 가 facade 로 호출 측에 단일 진입점을 제공한다. `tauriIpc` 와 `webIpc` 는 같은 인터페이스(`projectList`, `fileAddMany`, `pipelineRun`, …)를 노출해 store 와 컴포넌트가 surface 를 모르게 동작한다. 동일하게 `tauriOnPipelineEvent` / `webOnPipelineEvent` 가 백엔드 이벤트를 통일된 `(handler) => Promise<() => void>` 시그니처로 노출한다.

## 주요 의존 관계 / Key Dependencies
- `client.ts` → `env.isTauri` + 두 클라이언트.
- 양쪽 클라이언트 모두 `types.ts` 의 도메인 모델에 의존.
- `tauriClient` 는 `@tauri-apps/api/core`, `@tauri-apps/api/event` 정적 import.
- `webClient` 는 fetch / EventSource (브라우저 내장).

## 위험 신호 / Risk Signals
- IPC 응답 런타임 검증 부재 — 백엔드 변경 시 silent drift.
- 채널 이름 `pipeline://event` 가 magic string.
- web 번들에 Tauri 모듈이 포함되는 빌드 분리 부족 (번들 사이즈 패널티).
