# src/pages/HomePage.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 홈/인덱스 페이지 — 빈 프로젝트 생성 또는 파일 드롭으로 프로젝트 자동 생성 후 이동.
- 진입점에서의 거리 / Distance from entrypoint: 2 (App → Routes → here)
- 라인 수 / LOC: 90
- 주요 export / Exports: `HomePage`
- 의존 모듈 / Depends on (internal): `../components/{DropZone,icons}`, `../store/useAppStore`, `../lib/{filePicker,toast}`, `../hooks/useTauriDrop`
- 사용된 외부 패키지 / External packages used: `react-router-dom`
- 부수효과 / Side effects: createProject(IPC), addFiles(IPC), navigate
- 동시성 모델 / Concurrency model: async 핸들러 + try/catch
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/pages/HomePage.tsx lines=1-12
import { useNavigate } from "react-router-dom";
import { DropZone } from "../components/DropZone";
import { useAppStore } from "../store/useAppStore";
import { IconPlus, IconSparkles } from "../components/icons";
import { pickMediaFiles, type PickedMedia } from "../lib/filePicker";
import { useTauriDrop } from "../hooks/useTauriDrop";
import toast from "../lib/toast";

export function HomePage() {
  const navigate = useNavigate();
  const createProject = useAppStore((s) => s.createProject);
  const addFiles = useAppStore((s) => s.addFiles);
```

**분석 / Analysis (Level 2)**
- 적은 의존성 — 홈은 단순 진입점이라 깔끔.

```tsx path=src/pages/HomePage.tsx lines=14-31
  const handleCreate = async () => {
    try {
      const p = await createProject();
      navigate(`/projects/${p.id}`);
    } catch (e) {
      console.error("createProject failed", e);
      toast(`프로젝트 생성 실패: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const startWithPicked = async (picked: PickedMedia) => {
    const isEmpty =
      picked.mode === "paths" ? picked.inputs.length === 0 : picked.files.length === 0;
    if (isEmpty) return;
    const p = await createProject();
    await addFiles(p.id, picked);
    navigate(`/projects/${p.id}`);
  };
```

**분석 / Analysis (Level 2)**
- `handleCreate` 가 `await createProject()` 로 올바르게 처리 — Sidebar 의 `handleNewProject` 버그와 대비된다. 같은 store 액션을 다르게 다루므로 일관성 결여.
- `startWithPicked` 가 빈 픽 조기 반환 → 빈 프로젝트가 자동 생성되는 사고 방지.

**짚어볼 점 / Watch-outs**
- `startWithPicked` 가 try/catch 없음 — 호출자가 wrapping 해야 한다 (실제로 아래 핸들러들이 wrap).

```tsx path=src/pages/HomePage.tsx lines=33-58
  const handlePick = async () => {
    try {
      const picked = await pickMediaFiles();
      await startWithPicked(picked);
    } catch (e) {
      console.error("handlePick failed", e);
      toast(`파일 추가 실패: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleDropZoneFiles = async (picked: PickedMedia) => {
    try {
      await startWithPicked(picked);
    } catch (e) {
      console.error("dropzone files failed", e);
      toast(`파일 추가 실패: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  useTauriDrop(async (picked) => {
    try {
      await startWithPicked(picked);
    } catch (e) {
      console.error("drop create project failed", e);
    }
  });
```

**분석 / Analysis (Level 2)**
- `handlePick`/`handleDropZoneFiles` 가 사용자 토스트로 실패를 알린다 — ProjectPage 와 비교했을 때 더 친절.
- `useTauriDrop` 콜백은 toast 없음 → 다른 두 핸들러와 일관성 불일치.

**짚어볼 점 / Watch-outs**
- 일관성: 셋 다 같은 startWithPicked 를 부르지만 사용자 피드백 정책이 갈린다. 하나의 helper 로 합치는 게 안전.

```tsx path=src/pages/HomePage.tsx lines=60-89
  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">As It Sounds</h1>
      </header>

      <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center gap-8 px-6 py-10">
        <div className="space-y-2 text-center">
          <div className="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-2xl bg-brand-soft text-brand">
            <IconSparkles />
          </div>
          <h2 className="text-2xl font-semibold tracking-tight">
            영어 음성을 한글 발음 그대로 볼까요?
          </h2>
          <p className="text-sm text-fg-muted">
            새 프로젝트를 만들거나 파일을 바로 끌어다 놓아 시작하세요.
          </p>
        </div>

        <DropZone onFiles={handleDropZoneFiles} onPick={handlePick} />

        <div className="flex justify-center">
          <button className="btn-outline" onClick={handleCreate}>
            <IconPlus />
            빈 프로젝트로 시작
          </button>
        </div>
      </div>
    </div>
  );
}
```

**분석 / Analysis (Level 2)**
- 중앙 정렬된 hero + DropZone + 빈 프로젝트 버튼. 매우 단순한 메인 액션 페이지.
- DropZone 의 onFiles 가 올바르게 `handleDropZoneFiles` 로 연결 — ProjectPage 의 compact DropZone 버그와 대조됨.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 빈 픽 조기 반환으로 빈 프로젝트 자동 생성 사고 방지.
  - 토스트 사용자 피드백.
- 위험 / Risks:
  - useTauriDrop 콜백만 toast 없어 일관성 결여.
  - startWithPicked 가 try/catch 없이 호출자에게 위임.
- 개선 권고 / Recommendations:
  - 세 핸들러를 `withFeedback(startWithPicked, "파일 추가 실패")` 처럼 wrap 하여 일관성 확보.
- 재읽기 권고 레벨 / Suggested re-read level: 1.
