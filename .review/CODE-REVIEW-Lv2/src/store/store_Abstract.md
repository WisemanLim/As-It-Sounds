# store/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| useAppStore.ts | 187 | 앱 전역 상태 + IPC orchestration | [useAppStore.ts.md](./useAppStore.ts.md) |
| usePipelineStore.ts | 36 | 파일별 stage + 모델 다운로드 진행률 | [usePipelineStore.ts.md](./usePipelineStore.ts.md) |

## 디렉토리 구조 / Structure Summary
zustand 기반 두 스토어가 명확하게 책임을 분리한다. `useAppStore` 는 도메인 데이터(projects/files/segments) + IPC 트리거 액션을, `usePipelineStore` 는 일시적인 진행 상황(stages, download)을 보관한다. 도메인 데이터 mutation 은 항상 "IPC 호출 → set" 순서 — 백엔드를 single source of truth 로 둔다. PipelineListener 가 IPC 이벤트를 받아 두 스토어를 함께 갱신해 UI 동기화를 유지한다.

## 주요 의존 관계 / Key Dependencies
- `useAppStore` → `ipc/client`, `types`, `lib/filePicker.PickedMedia`.
- `usePipelineStore` → `zustand` 만.
- 거의 모든 컴포넌트가 두 스토어 중 하나 이상을 셀렉터로 구독.

## 위험 신호 / Risk Signals
- `useAppStore.setActiveProject` 의 비동기 fetch가 race condition 보호 없음 — 빠른 프로젝트 전환 시 stale 응답으로 잘못된 파일/세그먼트 세팅 가능.
- `detectKind` 가 store 모듈에 export 됨 — 의존 방향이 어색 (lib/media.ts 가 자연).
- `Stage` 타입이 `usePipelineStore` 와 `FileList.tsx` 양쪽에 중복 정의 — DRY 위반.
