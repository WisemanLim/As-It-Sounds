# `src/` 디렉터리 요약 / Directory Abstract

> [← INDEX.md](../INDEX.md)

## 파일 목록 / Files

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| main.tsx | 54 | [main.tsx.md](./main.tsx.md) |
| App.tsx | 61 | [App.tsx.md](./App.tsx.md) |
| types.ts | 62 | [types.ts.md](./types.ts.md) |

## 구조 요약

`src/`는 React 19 + Vite SPA의 루트. `main.tsx`가 DOM에 마운트하고, `App.tsx`가 HashRouter 안에서 `bootstrap()` 게이트 후 라우트를 렌더한다. `types.ts`는 백엔드 `core/models.rs`와 1:1 매핑되는 도메인 타입 모음.

## 핵심 의존

- `react`, `react-dom`, `react-router-dom`
- 내부: `./components/*`, `./pages/*`, `./store/useAppStore`

## 리스크 시그널

- `main.tsx`의 fatal overlay는 inline CSS string 사용 — 보안상 큰 문제는 없으나 i18n/스타일링 일관성과 멀어짐.
- `App.tsx`의 bootstrap이 실패하면 IPC 전체가 죽는 단일 게이트라, 부분 실패에 회복력이 없음.
