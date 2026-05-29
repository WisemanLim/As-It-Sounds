# src/pages/ProjectPage.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 단일 프로젝트의 메인 작업 화면 — 파일 드롭/픽, 활성 파일 미디어 재생, 세그먼트 표시, 파이프라인 실행.
- 진입점에서의 거리 / Distance from entrypoint: 2 (App → Routes → here)
- 라인 수 / LOC: 218
- 주요 export / Exports: `ProjectPage`
- 의존 모듈 / Depends on (internal): `../components/{DropZone,FileList,MediaPlayer,SegmentView,icons}`, `../store/{useAppStore,usePipelineStore}`, `../ipc/client`, `../lib/filePicker`, `../hooks/useTauriDrop`
- 사용된 외부 패키지 / External packages used: `react`, `react-router-dom`
- 부수효과 / Side effects: setActiveProject(IPC fetch), addFiles(IPC), pipelineRun(IPC), renameProject(IPC)
- 동시성 모델 / Concurrency model: 여러 async 핸들러 (try/catch + console.error 패턴)
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/pages/ProjectPage.tsx lines=1-13
import { useEffect, useMemo, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { DropZone } from "../components/DropZone";
import { FileList } from "../components/FileList";
import { MediaPlayer, type MediaPlayerHandle } from "../components/MediaPlayer";
import { SegmentView } from "../components/SegmentView";
import { IconPlus, IconSparkles } from "../components/icons";
import { useAppStore } from "../store/useAppStore";
import { usePipelineStore } from "../store/usePipelineStore";
import { ipc } from "../ipc/client";
import { pickMediaFiles } from "../lib/filePicker";
import { useTauriDrop } from "../hooks/useTauriDrop";
```

**분석 / Analysis (Level 2)**
- 이 페이지가 사실상 앱의 핵심 — 가장 많은 의존성. 컴포지션이 깔끔하면 확장이 쉬워진다.

```tsx path=src/pages/ProjectPage.tsx lines=14-29
export function ProjectPage() {
  const { id } = useParams<{ id: string }>();

  // Only select stable top-level state from the store; derive locally.
  const projects = useAppStore((s) => s.projects);
  const allFiles = useAppStore((s) => s.files);
  const allSegments = useAppStore((s) => s.segments);
  const activeFileId = useAppStore((s) => s.activeFileId);

  const addFiles = useAppStore((s) => s.addFiles);
  const setActiveFile = useAppStore((s) => s.setActiveFile);
  const setActiveProject = useAppStore((s) => s.setActiveProject);
  const renameProject = useAppStore((s) => s.renameProject);

  const stages = usePipelineStore((s) => s.stages);
  const download = usePipelineStore((s) => s.download);
```

**분석 / Analysis (Level 2)**
- 주석이 디자인 결정을 잘 설명 — "전역에서 stable한 슬라이스만 select, derive 는 로컬에서".
- 각 셀렉터를 분리해 리렌더 범위를 최소화.

**짚어볼 점 / Watch-outs**
- `allFiles`/`allSegments` 가 모든 프로젝트의 파일을 보유 → 다른 프로젝트 파일이 바뀌어도 이 페이지가 리렌더된다. 큰 데이터셋에서 성능 부담.

```tsx path=src/pages/ProjectPage.tsx lines=31-56
  const project = useMemo(
    () => projects.find((p) => p.id === id) ?? null,
    [projects, id],
  );
  const files = useMemo(
    () => allFiles.filter((f) => f.projectId === id),
    [allFiles, id],
  );
  const sortedFiles = useMemo(
    () => [...files].sort((a, b) => a.createdAt - b.createdAt),
    [files],
  );
  const activeFile = useMemo(
    () =>
      allFiles.find((f) => f.id === activeFileId && f.projectId === id) ?? null,
    [allFiles, activeFileId, id],
  );
  const segments = useMemo(
    () =>
      allSegments
        .filter((seg) => seg.fileId === activeFile?.id)
        .slice()
        .sort((a, b) => a.index - b.index),
    [allSegments, activeFile?.id],
  );
```

**분석 / Analysis (Level 2)**
- 모든 파생 데이터를 useMemo 로 메모이즈 — 깔끔.
- `.slice()` 로 sort 전에 복사 → 원본 mutate 안전. 한 번은 `[...files]` 다른 곳은 `.slice()` — 표현 일관성은 부족하지만 동일 효과.

```tsx path=src/pages/ProjectPage.tsx lines=57-83
  const [currentTimeMs, setCurrentTimeMs] = useState(0);
  const playerRef = useRef<MediaPlayerHandle>(null);

  useEffect(() => {
    if (!id) return;
    setActiveProject(id).catch((e) => {
      console.error("setActiveProject failed", e);
    });
  }, [id, setActiveProject]);

  useTauriDrop(async (picked) => {
    if (!id) return;
    try {
      const created = await addFiles(id, picked);
      if (created.length > 0) await setActiveFile(created[0].id);
    } catch (e) {
      console.error("drop add files failed", e);
    }
  });

  if (!project) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-fg-muted">
        프로젝트를 찾을 수 없습니다.
      </div>
    );
  }
```

**분석 / Analysis (Level 2)**
- `useEffect` 가 라우트 id 변경마다 setActiveProject 호출 → 파일/세그먼트 fetch 트리거. 직접 사이드 효과는 store 에 위임.
- `useTauriDrop` 가 컴포넌트 마운트 동안 활성. id 가 바뀌면 콜백이 새로워지고 useTauriDrop 내부 ref 가 업데이트.
- 프로젝트가 없으면 일찍 return → 깨끗한 폴백.

**짚어볼 점 / Watch-outs**
- `useTauriDrop` 의 effect 가 mount 시 한 번만 실행 → id 가 바뀌어도 listener 재등록은 일어나지 않지만, `onDropRef.current` 가 최신 id 클로저를 잡으므로 동작은 OK (ref 패턴의 장점).

```tsx path=src/pages/ProjectPage.tsx lines=85-118
  const handlePick = async () => {
    try {
      const picked = await pickMediaFiles();
      const created = await addFiles(project.id, picked);
      if (created.length > 0 && !activeFileId) {
        await setActiveFile(created[0].id);
      }
    } catch (e) {
      console.error("handlePick failed", e);
    }
  };

  const handleDropZoneFiles = async (picked: import("../lib/filePicker").PickedMedia) => {
    try {
      const created = await addFiles(project.id, picked);
      if (created.length > 0 && !activeFileId) {
        await setActiveFile(created[0].id);
      }
    } catch (e) {
      console.error("handleDropZoneFiles failed", e);
    }
  };

  const runPipeline = async () => {
    try {
      const ids = sortedFiles
        .filter((f) => f.status !== "processing")
        .map((f) => f.id);
      if (ids.length === 0) return;
      await ipc.pipelineRun(ids);
    } catch (e) {
      console.error("pipelineRun failed", e);
    }
  };
```

**분석 / Analysis (Level 2)**
- `handlePick`/`handleDropZoneFiles` 가 거의 동일 — 차이는 picker 호출 유무. DRY 위반은 작지만 `if (created.length > 0 && !activeFileId)` 패턴이 두 곳에 중복.
- inline type import (`import("../lib/filePicker").PickedMedia`) — 인라인 사용은 약간 무겁다. 일반 import 로 옮기는 게 깔끔.
- `pipelineRun` 이 toast 같은 사용자 피드백 없이 console.error 만 한다 → 사용자가 실패를 인지하기 어려움.

**짚어볼 점 / Watch-outs**
- `runPipeline` 실패 시 사용자 토스트 없음. PipelineListener 가 fileError 이벤트는 잡지만, `pipeline_run` 명령 자체가 실패하면 silent.

```tsx path=src/pages/ProjectPage.tsx lines=119-152
  const mediaSrc = activeFile ? ipc.mediaSrc(activeFile) : null;

  const hasFiles = sortedFiles.length > 0;
  const anyProcessing = sortedFiles.some((f) => f.status === "processing");
  const activeStage = activeFile ? stages[activeFile.id] : null;

  return (
    <div className="flex h-full flex-col">
      <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b border-line px-6">
        <input
          className="min-w-0 flex-1 truncate bg-transparent text-sm font-semibold text-fg placeholder:text-fg-subtle focus:outline-none"
          value={project.name}
          onChange={(e) =>
            renameProject(project.id, e.target.value).catch((err) =>
              console.error("rename failed", err),
            )
          }
        />
        <div className="flex items-center gap-2">
          <button className="btn-outline" onClick={handlePick}>
            <IconPlus />
            파일 추가
          </button>
          <button
            className="btn-primary"
            onClick={runPipeline}
            disabled={!hasFiles || anyProcessing}
          >
            <IconSparkles />
            {anyProcessing ? "처리 중..." : "변환 시작"}
          </button>
        </div>
      </header>
```

**분석 / Analysis (Level 2)**
- `<input value={project.name}>` 가 매 키 입력마다 renameProject 호출 → 백엔드에 매 키스트로크마다 IPC. SettingsPage 와 동일한 디바운스 부재.
- 변환 시작 버튼이 anyProcessing 일 때 disabled — 동시 실행 방지.

**짚어볼 점 / Watch-outs**
- 이름 변경 디바운스 부재.

```tsx path=src/pages/ProjectPage.tsx lines=153-216
      {download && (
        <div className="border-b border-line bg-brand-soft/30 px-6 py-2 text-xs text-brand">
          {download.message}{" "}
          {download.progress > 0 &&
            `· ${Math.round(download.progress * 100)}%`}
          <div className="mt-1 h-1 w-full overflow-hidden rounded bg-bg-hover">
            <div
              className="h-full bg-brand transition-all"
              style={{ width: `${Math.round(download.progress * 100)}%` }}
            />
          </div>
        </div>
      )}

      {!hasFiles ? (
        <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center px-6 py-10">
          <DropZone onFiles={handleDropZoneFiles} onPick={handlePick} />
        </div>
      ) : (
        <div className="grid min-h-0 flex-1 grid-cols-[minmax(280px,340px)_1fr] gap-4 overflow-hidden p-4">
          <aside className="flex min-h-0 flex-col gap-3 overflow-y-auto pr-1">
            <DropZone onFiles={handlePick} onPick={handlePick} compact />
            <FileList files={sortedFiles} stages={stages} />
          </aside>

          <section className="flex min-h-0 flex-col gap-3 overflow-hidden">
            {activeFile && mediaSrc ? (
              <MediaPlayer
                ref={playerRef}
                file={activeFile}
                src={mediaSrc}
                onTimeUpdate={setCurrentTimeMs}
                onDurationChange={() => {
                  /* backend sets duration during pipeline */
                }}
              />
            ) : (
              <div className="...">좌측에서 파일을 선택하세요.</div>
            )}

            {activeStage && (
              <div className="...">
                <span className="...">{activeStage.stage}</span>
                {activeStage.message}
              </div>
            )}

            <div className="min-h-0 flex-1 overflow-y-auto pr-1">
              <SegmentView
                segments={segments}
                currentTimeMs={currentTimeMs}
                onSeek={(ms) => playerRef.current?.seek(ms)}
              />
            </div>
          </section>
        </div>
      )}
    </div>
  );
}
```

**분석 / Analysis (Level 2)**
- **버그 의심**: `<DropZone onFiles={handlePick} ... />` 에서 onFiles 콜백이 `handlePick` 으로 잘못 연결되어 있다. handlePick 는 `() => Promise<void>` 인데 onFiles 는 `(picked: PickedMedia) => void` 시그니처. 타입은 통과(인자 무시)되지만 의도는 분명 `handleDropZoneFiles`. → compact 사이드바의 DropZone 에 드롭하면 picker 가 뜨고 드롭된 파일은 무시됨.
- `download` UI 가 모델 다운로드 진행률을 보여주는 자리. progress 0 일 때 % 표시는 숨기는 등 세심.
- 빈/있음 분기로 화면 레이아웃이 완전히 다름 → DropZone 이 페이지 중앙 vs 사이드바로 이동.

**짚어볼 점 / Watch-outs**
- **버그**: sidebar compact DropZone 의 onFiles 가 잘못된 함수.
- `onDurationChange` 콜백이 비어 있음 — 백엔드가 duration 을 갱신하므로 UI 가 별도로 가질 필요는 없지만, 활성 파일의 duration 이 백엔드 처리 전까진 누락 (FileList 에서 `--:--` 로 표시).

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 셀렉터 분리 + useMemo 파생으로 리렌더 범위 관리.
  - 빈/있음 레이아웃 분기가 깔끔.
  - 모든 async 호출에 try/catch.
- 위험 / Risks:
  - **버그**: compact `<DropZone onFiles={handlePick}/>` — 드롭된 파일이 무시됨.
  - 이름/엔드포인트 디바운스 부재 → 매 키스트로크마다 IPC.
  - `pipelineRun` 실패가 사용자에게 안 알려짐 (console only).
- 개선 권고 / Recommendations:
  - `onFiles={handleDropZoneFiles}` 로 수정.
  - 이름 입력에 debounce(300ms) 적용.
  - `runPipeline` catch 안에 `toast("변환 시작 실패: ...")` 추가.
- 재읽기 권고 레벨 / Suggested re-read level: 2 — 버그/디바운스 수정 후 재검토.
