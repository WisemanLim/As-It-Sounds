# `src/components/` 디렉터리 요약

> [← INDEX.md](../../INDEX.md)

## 파일 목록

| 파일 | 줄 수 | 리뷰 |
|------|-----:|------|
| Layout.tsx | 13 | [Layout.tsx.md](./Layout.tsx.md) |
| Sidebar.tsx | 181 | [Sidebar.tsx.md](./Sidebar.tsx.md) |
| DropZone.tsx | 92 | [DropZone.tsx.md](./DropZone.tsx.md) |
| ErrorBoundary.tsx | 45 | [ErrorBoundary.tsx.md](./ErrorBoundary.tsx.md) |
| FileList.tsx | 120 | [FileList.tsx.md](./FileList.tsx.md) |
| MediaPlayer.tsx | 91 | [MediaPlayer.tsx.md](./MediaPlayer.tsx.md) |
| PipelineListener.tsx | 59 | [PipelineListener.tsx.md](./PipelineListener.tsx.md) |
| SegmentView.tsx | 73 | [SegmentView.tsx.md](./SegmentView.tsx.md) |
| icons.tsx | 110 | [icons.tsx.md](./icons.tsx.md) |

## 구조 요약

UI 컴포넌트 모음. Layout이 Sidebar+Outlet으로 셸을 그리고, 페이지에서 DropZone/FileList/MediaPlayer/SegmentView를 조합한다. PipelineListener는 화면 그리지 않는 부수효과 컴포넌트로 백엔드 이벤트→store를 잇는다.

## 핵심 의존

- React (hooks/forwardRef)
- 내부: `../store/*`, `../lib/cn`, `../lib/format`, `../ipc/client`

## 리스크 시그널

- ~~`Sidebar.tsx::handleNewProject`가 Promise를 navigate 인자에 즉시 보간 — URL이 깨짐~~ ✅ 수정됨 2026-05-30 (`async`/`await` + `p.id`; INDEX §4.3 #3).
- `PipelineListener`의 cleanup 레이스 (INDEX §4.3 #4).
- `MediaPlayer.tsx`가 `audioRef.current?.currentTime`을 렌더 함수 안에서 직접 읽음 → 첫 렌더와 이후 렌더 사이의 동기화가 보장되지 않음.
