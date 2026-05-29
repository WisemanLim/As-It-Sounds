# src/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| App.tsx | 62 | 최상위 React 컴포넌트, 부트스트랩 + 라우트 | [App.tsx.md](./App.tsx.md) |
| main.tsx | 55 | 진입점, React root + fatal 오버레이 | [main.tsx.md](./main.tsx.md) |
| types.ts | 63 | 도메인 모델 + IPC 페이로드 타입 | [types.ts.md](./types.ts.md) |
| components/ | — | UI 컴포넌트 | [components_Abstract.md](./components/components_Abstract.md) |
| hooks/ | — | 커스텀 훅 | [hooks_Abstract.md](./hooks/hooks_Abstract.md) |
| ipc/ | — | desktop/web IPC 어댑터 | [ipc_Abstract.md](./ipc/ipc_Abstract.md) |
| lib/ | — | 유틸 (env, cn, format, toast, filePicker) | [lib_Abstract.md](./lib/lib_Abstract.md) |
| mock/ | — | 더미 데이터 | [mock_Abstract.md](./mock/mock_Abstract.md) |
| pages/ | — | 라우트 단위 페이지 | [pages_Abstract.md](./pages/pages_Abstract.md) |
| store/ | — | zustand 전역 상태 | [store_Abstract.md](./store/store_Abstract.md) |

## 디렉토리 구조 / Structure Summary
`src/` 는 React + TypeScript SPA의 표준 레이아웃이다. `main.tsx` 가 HashRouter 와 ErrorBoundary 를 거쳐 `App.tsx` 를 부팅하고, `App.tsx` 가 `Layout` + `Routes` 로 페이지를 조립한다. 모든 도메인 모델은 `types.ts` 단일 출처에서 흘러나오고, `store/` 의 두 zustand 스토어가 IPC 어댑터(`ipc/client.ts`)를 통해 백엔드와 통신한다. 데스크톱(Tauri)과 웹은 `lib/env.ts` 의 `isTauri` 플래그로 분기되며, 같은 React 번들이 양쪽 surface 에 그대로 출하된다.

## 주요 의존 관계 / Key Dependencies
- `App.tsx` → `Layout` + `Routes` + `PipelineListener` + `ErrorBoundary` + `useAppStore.bootstrap`
- `pages/*` → `store/*` + `ipc/client` + `components/*` + `lib/*`
- `store/useAppStore` → `ipc/client` (모든 mutation은 IPC 후 set)
- `ipc/client` → `tauriClient` 또는 `webClient` (isTauri 분기)
- `components/PipelineListener` → `ipc/client.onPipelineEvent` + 두 store

## 위험 신호 / Risk Signals
- `Settings.translateApiKey` 가 평문 string 으로 store/UI/IPC 페이로드를 떠다님 — 마스킹/저장소 보안 검토 필요.
- 미디어 확장자 정의가 `useTauriDrop.ts`/`filePicker.ts`/`useAppStore.ts(detectKind)` 등 3~4 곳에 중복 — drift 위험.
- 다수의 async 액션이 race condition / stale-write 보호 없음 (특히 `setActiveProject`, `PipelineListener` subscribe Promise).
