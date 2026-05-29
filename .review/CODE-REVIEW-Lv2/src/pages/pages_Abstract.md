# pages/ — 디렉토리 개요 / Directory Abstract

> [← INDEX.md](../../INDEX.md)

## 파일 목록 / File List
| 파일 / File | LOC | 역할 / Role | 리뷰 링크 / Review |
|---|---|---|---|
| HomePage.tsx | 90 | 홈 — 빈 프로젝트 생성 또는 드롭으로 자동 생성 | [HomePage.tsx.md](./HomePage.tsx.md) |
| ProjectPage.tsx | 218 | 프로젝트 핵심 작업 화면 | [ProjectPage.tsx.md](./ProjectPage.tsx.md) |
| SettingsPage.tsx | 156 | Whisper 모델/번역/ffmpeg 설정 | [SettingsPage.tsx.md](./SettingsPage.tsx.md) |

## 디렉토리 구조 / Structure Summary
react-router 의 라우트 단위 페이지를 모은다. `HomePage` 는 hero + DropZone + 빈 프로젝트 생성 버튼, `ProjectPage` 는 사이드 파일 리스트 + 메인 미디어 재생기 + 세그먼트 뷰의 3분할, `SettingsPage` 는 form 기반 설정 페이지. 모두 store + ipc client + lib 유틸을 조합한다.

## 주요 의존 관계 / Key Dependencies
- `HomePage`/`ProjectPage` → `useAppStore`, `lib/filePicker`, `hooks/useTauriDrop`, `components/DropZone`.
- `ProjectPage` → `usePipelineStore` (stages, download), `components/{FileList, MediaPlayer, SegmentView}`.
- `SettingsPage` → `ipc/client.settingsGet/Set`, `lib/toast`.

## 위험 신호 / Risk Signals
- `ProjectPage` 사이드바의 compact DropZone 이 `onFiles={handlePick}` 으로 잘못 연결 → 드롭된 파일이 picker 로 우회되어 결과 무시.
- `SettingsPage` API 키 input 이 `type="text"` (평문 노출).
- 입력 컨트롤(이름, endpoint, API key, ffmpeg path) 모두 디바운스 부재 — 매 keystroke 마다 IPC 호출.
