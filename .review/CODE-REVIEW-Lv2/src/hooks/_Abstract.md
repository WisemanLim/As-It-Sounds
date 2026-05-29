# `src/hooks/` 디렉터리 요약

> [← INDEX.md](../../INDEX.md)

## 파일 목록

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| useTauriDrop.ts | 111 | [useTauriDrop.ts.md](./useTauriDrop.ts.md) |

## 구조 요약

Tauri 웹뷰의 네이티브 드래그 드롭 이벤트를 구독해 React 콜백으로 매핑하는 단일 훅. 웹 모드에서는 no-op.

## 핵심 의존

- `@tauri-apps/api/webview` (dynamic import)
- `@tauri-apps/plugin-fs::stat` (dynamic import)
- 내부: `../lib/env`, `../store/useAppStore::detectKind`

## 리스크 시그널

- 이벤트 등록 자체가 비동기인데 빠른 unmount 시 race가 있음 → `disposed` 플래그로 처리하지만 완벽하지 않음.
- `useEffect`의 deps가 `[]`라서 onDrop/onHover 변경 시 ref만 갱신되는 점은 의도적이지만 주석이 더 친절해야 함.
