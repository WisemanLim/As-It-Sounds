# hooks/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| useTauriDrop.ts | 111 | Tauri webview 윈도우-레벨 파일 드롭 훅 (web 에선 no-op) | [useTauriDrop.ts.md](./useTauriDrop.ts.md) |

## 디렉토리 구조 / Structure Summary
현재 단일 훅만 있다. `useTauriDrop` 은 Tauri 환경에서 webview 의 글로벌 드래그/드롭 이벤트를 듣고 콜백으로 전달한다. 웹에서는 `isTauri` 가드로 즉시 return — 동일 React 코드가 두 surface 에서 안전하게 동작하도록 보장한다.

## 주요 의존 관계 / Key Dependencies
- `../lib/env.isTauri` 분기.
- 동적 import 로 `@tauri-apps/plugin-fs`, `@tauri-apps/api/webview` 로드.
- `../store/useAppStore.detectKind` 를 사용 (위치 부적절 — `lib/media.ts` 가 더 적절).

## 위험 신호 / Risk Signals
- EXTS / baseName / detectKind 가 `lib/filePicker.ts` 와 중복 — drift 위험.
