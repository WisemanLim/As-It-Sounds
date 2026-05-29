# `src/ipc/` 디렉터리 요약

> [← INDEX.md](../../INDEX.md)

## 파일 목록

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| client.ts | 16 | [client.ts.md](./client.ts.md) |
| tauriClient.ts | 54 | [tauriClient.ts.md](./tauriClient.ts.md) |
| webClient.ts | 121 | [webClient.ts.md](./webClient.ts.md) |

## 구조 요약

런타임 분기로 데스크톱(tauri invoke)과 웹(fetch+SSE)을 골라 동일한 `ipc` 인터페이스를 노출한다. 페이지/스토어는 `ipc`만 보면 됨.

## 핵심 의존

- `@tauri-apps/api/core`, `@tauri-apps/api/event` (desktop)
- `EventSource`, `fetch` (web)
- 내부: `../lib/env::isTauri`, `../types`

## 리스크 시그널

- `webClient.fileAddMany`와 `tauriClient.fileUpload`가 서로 상대 모드에서 throw — 호출 측 분기 누락 시 즉시 깨짐(현재 `useAppStore.addFiles`에서 잘 분기).
- `webClient.webOnPipelineEvent`가 `EventSource`의 영구 실패를 사용자에게 알리지 않음(콘솔 debug만).
- `webClient` 의 `jsend` 가 `Promise<T | void>`라 호출자가 캐스팅 필요 — 타입 안전성 떨어짐.
