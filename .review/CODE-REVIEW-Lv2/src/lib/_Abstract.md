# `src/lib/` 디렉터리 요약

> [← INDEX.md](../../INDEX.md)

## 파일 목록

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| cn.ts | 6 | [cn.ts.md](./cn.ts.md) |
| env.ts | 21 | [env.ts.md](./env.ts.md) |
| filePicker.ts | 98 | [filePicker.ts.md](./filePicker.ts.md) |
| format.ts | 36 | [format.ts.md](./format.ts.md) |
| toast.ts | 14 | [toast.ts.md](./toast.ts.md) |

## 구조 요약

순수 유틸리티 모음. cn=클래스 머지, env=Tauri 감지/API base, filePicker=Tauri vs 웹 분기, format=시간/상대시간, toast=DOM 직접 삽입.

## 핵심 의존

- `clsx`, `tailwind-merge`
- `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-fs` (dynamic import)
- 내부: `../store/useAppStore::detectKind` (역방향 의존)

## 리스크 시그널

- `filePicker.ts → useAppStore`: lib이 store를 의존하는 역방향(INDEX §4.2).
- `toast.ts`: 글로벌 DOM mutation만 — React 트리 밖. 컴포넌트 라이프사이클에서 청소 안 됨(자동 4초 후 remove).
- `env.ts`의 `apiBaseUrl()`이 매 호출마다 import.meta.env를 재평가 — 큰 비용은 아님.
