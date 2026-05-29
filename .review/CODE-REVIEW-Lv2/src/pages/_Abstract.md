# `src/pages/` 디렉터리 요약

> [← INDEX.md](../../INDEX.md)

## 파일 목록

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| HomePage.tsx | 90 | [HomePage.tsx.md](./HomePage.tsx.md) |
| ProjectPage.tsx | 217 | [ProjectPage.tsx.md](./ProjectPage.tsx.md) |
| SettingsPage.tsx | 155 | [SettingsPage.tsx.md](./SettingsPage.tsx.md) |

## 구조 요약

라우트 진입점 3종. Home은 새 프로젝트 시작+파일 드롭, Project는 메인 작업 화면(파일 리스트/플레이어/세그먼트), Settings는 폼이다.

## 핵심 의존

- `../store/useAppStore`, `../store/usePipelineStore`, `../ipc/client`
- `../lib/filePicker`, `../hooks/useTauriDrop`
- `../components/*`

## 리스크 시그널

- `ProjectPage` 프로젝트명 input의 onChange가 모든 키 입력마다 `renameProject(ipc.invoke)` 호출 → 네트워크/IO 과다. debounce 권장.
- `SettingsPage`도 select onChange 마다 즉시 `settingsSet` (저장 폭주).
- `HomePage.handleCreate`/`startWithPicked`이 createProject Promise를 사용하나, 동일한 store 메서드를 호출하므로 atomic 보장은 store에 맡김(현재 atomic 아님).
