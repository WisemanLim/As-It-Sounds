# components/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| DropZone.tsx | 93 | 드래그&드롭 + 클릭 파일 picker 트리거 | [DropZone.tsx.md](./DropZone.tsx.md) |
| PipelineListener.tsx | 60 | 백엔드 파이프라인 이벤트를 store 에 반영 | [PipelineListener.tsx.md](./PipelineListener.tsx.md) |
| FileList.tsx | 121 | 프로젝트 내 미디어 파일 리스트 | [FileList.tsx.md](./FileList.tsx.md) |
| Layout.tsx | 13 | Sidebar + Outlet 2단 그리드 | [Layout.tsx.md](./Layout.tsx.md) |
| icons.tsx | 110 | 인라인 SVG 아이콘 모음 | [icons.tsx.md](./icons.tsx.md) |
| Sidebar.tsx | 181 | 프로젝트 목록/검색/생성/삭제 + 설정 진입 | [Sidebar.tsx.md](./Sidebar.tsx.md) |
| SegmentView.tsx | 73 | EN/한글음차/번역 세그먼트 리스트 + 시크 | [SegmentView.tsx.md](./SegmentView.tsx.md) |
| MediaPlayer.tsx | 92 | audio/video 통합 재생기 + seek API | [MediaPlayer.tsx.md](./MediaPlayer.tsx.md) |
| ErrorBoundary.tsx | 45 | 렌더 에러 폴백 + 재시도 | [ErrorBoundary.tsx.md](./ErrorBoundary.tsx.md) |

## 디렉토리 구조 / Structure Summary
프레젠테이션 컴포넌트와 부수효과 컴포넌트가 한 폴더에 섞여 있다. `DropZone`/`FileList`/`SegmentView`/`MediaPlayer`/`Sidebar` 는 UI를 그리는 일반 컴포넌트, `PipelineListener` 는 DOM 을 그리지 않는 "invisible" effect 컴포넌트, `ErrorBoundary` 는 클래스 컴포넌트로 fallback UI 를 담당한다. `icons.tsx` 는 외부 의존성을 피하기 위한 in-house 아이콘 세트, `Layout` 은 사이드바 + Outlet 의 얇은 셸이다.

## 주요 의존 관계 / Key Dependencies
- 대부분 컴포넌트가 `lib/cn` + `lib/format` + `icons` 를 공유.
- `Sidebar`/`PipelineListener`/`FileList` 가 두 store(`useAppStore`, `usePipelineStore`) 의 액션을 직접 호출.
- `MediaPlayer` 는 `MediaPlayerHandle` 을 forwardRef + useImperativeHandle 로 부모(ProjectPage)에게 노출.

## 위험 신호 / Risk Signals
- ~~`Sidebar.handleNewProject` 가 `createProject()` 의 Promise 를 await 없이 URL 에 보간 → 라우트 깨짐~~ ✅ 수정됨 2026-05-30 (`async`/`await` + `p.id`).
- `MediaPlayer.play()` 의 Promise rejection 미처리 — autoplay 정책에서 fatal 오버레이 트리거 가능.
- `PipelineListener` 의 onPipelineEvent subscribe Promise 가 race condition 에 무방비 — `useTauriDrop` 의 `disposed` 패턴과 일관성 결여.
