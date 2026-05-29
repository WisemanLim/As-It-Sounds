# `src/store/` 디렉터리 요약

> [← INDEX.md](../../INDEX.md)

## 파일 목록

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| useAppStore.ts | 187 | [useAppStore.ts.md](./useAppStore.ts.md) |
| usePipelineStore.ts | 35 | [usePipelineStore.ts.md](./usePipelineStore.ts.md) |

## 구조 요약

Zustand 기반 글로벌 상태. `useAppStore`는 프로젝트/파일/세그먼트 + 액션(IPC 호출 포함), `usePipelineStore`는 파이프라인 진행 상태 전용.

## 핵심 의존

- `zustand`
- 내부: `../ipc/client`, `../types`, `../lib/filePicker`

## 리스크 시그널

- 액션이 IPC를 직접 호출 — 테스트 시 IPC mock 필요. 추후 액션을 분리하거나 의존 주입 권장.
- `setActiveProject`에서 두 번 set 호출 → 중간 상태에서 segments가 빈 배열로 잠깐 노출됨(깜빡임).
- `upsertSegment`가 O(N) 필터 후 append — 세그먼트가 많아지면 비용 누적.
