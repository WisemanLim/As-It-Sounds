# src/components/FileList.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 役할 / Role: 프로젝트의 미디어 파일 리스트 렌더링 + active/제거 인터랙션.
- 진입점에서의 거리 / Distance from entrypoint: 3 (App → ProjectPage → here)
- 라인 수 / LOC: 121
- 주요 export / Exports: `FileList`
- 의존 모듈 / Depends on (internal): `../lib/cn`, `../lib/format`, `../store/useAppStore`, `../types`, `./icons`
- 사용된 외부 패키지 / External packages used: `react` (구조적 import 없음 — 자식 컴포넌트 형태)
- 부수효과 / Side effects: store 의 `setActiveFile`/`removeFile` 호출
- 동시성 모델 / Concurrency model: 동기 이벤트 핸들러
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/components/FileList.tsx lines=1-15
import { cn } from "../lib/cn";
import { formatDuration } from "../lib/format";
import { useAppStore } from "../store/useAppStore";
import type { MediaFile } from "../types";
import { IconAudio, IconTrash, IconVideo } from "./icons";

interface Stage {
  stage: string;
  message: string;
}

interface Props {
  files: MediaFile[];
  stages?: Record<string, Stage>;
}
```

**분석 / Analysis (Level 2)**
- `Stage` 인터페이스가 여기와 `PipelineListener` 양쪽에 중복 정의되어 있다(`usePipelineStore`에도 있다). DRY 위반 — `usePipelineStore`에서 export 해 공유하면 깔끔.
- `files`는 부모에서 미리 필터링된 상태로 들어온다 (ProjectPage).

```tsx path=src/components/FileList.tsx lines=17-29
const statusStyle: Record<MediaFile["status"], string> = {
  pending: "bg-bg-hover text-fg-muted",
  processing: "bg-brand-soft text-brand",
  done: "bg-emerald-950 text-emerald-300",
  error: "bg-red-950 text-red-300",
};

const statusLabel: Record<MediaFile["status"], string> = {
  pending: "대기",
  processing: "처리 중",
  done: "완료",
  error: "오류",
};
```

**분석 / Analysis (Level 2)**
- `Record<MediaFile["status"], string>` 타입 — `FileStatus` 가 늘면 컴파일 에러로 누락을 잡는다. exhaustive table 패턴.
- 라벨/스타일을 분리하면서도 키가 일치 → 안전한 룩업.

```tsx path=src/components/FileList.tsx lines=31-39
export function FileList({ files, stages }: Props) {
  const activeFileId = useAppStore((s) => s.activeFileId);
  const setActiveFile = useAppStore((s) => s.setActiveFile);
  const removeFile = useAppStore((s) => s.removeFile);

  if (files.length === 0) {
    return null;
  }
```

**분석 / Analysis (Level 2)**
- 빈 상태는 부모(ProjectPage) 에서 DropZone 으로 분기하므로 여기는 null. 책임 분리 OK.

```tsx path=src/components/FileList.tsx lines=40-67
  return (
    <ul className="flex flex-col gap-1">
      {files.map((f) => {
        const isActive = f.id === activeFileId;
        const Icon = f.kind === "video" ? IconVideo : IconAudio;
        const stage = stages?.[f.id];
        return (
          <li key={f.id}>
            <div
              onClick={() => setActiveFile(f.id)}
              className={cn(
                "group flex cursor-pointer items-center gap-3 rounded-xl border px-3 py-2.5 transition-colors",
                isActive
                  ? "border-brand/50 bg-brand-soft/30"
                  : "border-line bg-bg-elevated hover:bg-bg-hover",
              )}
              title={f.path}
            >
              <div
                className={cn(
                  "flex h-9 w-9 shrink-0 items-center justify-center rounded-lg",
                  f.kind === "video"
                    ? "bg-fuchsia-950/40 text-fuchsia-300"
                    : "bg-brand-soft text-brand",
                )}
              >
                <Icon width={18} height={18} />
              </div>
```

**분석 / Analysis (Level 2)**
- `Icon` 변수를 동적 컴포넌트 참조로 사용 — JSX에서 `<Icon ... />` 처럼 대문자 변수만 컴포넌트로 인식된다. 자주 실수하는 부분(`<icon ... />` 은 HTML element로 취급됨).
- `title={f.path}` 로 호버 시 절대 경로 노출 — desktop에서는 OK지만, web에서는 서버측 경로가 사용자에게 노출될 수 있다(정보 노출, CWE-200 수준 낮음).
- `<div onClick={...}>` 도 row 단위 클릭 → DropZone과 마찬가지로 접근성/키보드 보강 여지.

**짚어볼 점 / Watch-outs**
- 정보 노출: `path`는 desktop에서는 사용자 본인 파일 시스템이지만, web 백엔드의 내부 경로가 들어가면 노출.

```tsx path=src/components/FileList.tsx lines=69-93
              <div className="min-w-0 flex-1">
                <div className="truncate text-sm font-medium text-fg">
                  {f.name}
                </div>
                <div className="mt-0.5 flex items-center gap-2 text-xs text-fg-subtle">
                  <span>{formatDuration(f.durationMs)}</span>
                  {f.sizeBytes != null && (
                    <>
                      <span>·</span>
                      <span>{(f.sizeBytes / (1024 * 1024)).toFixed(1)} MB</span>
                    </>
                  )}
                  {stage && (
                    <>
                      <span>·</span>
                      <span className="truncate text-brand">{stage.message}</span>
                    </>
                  )}
                </div>
                {f.error && (
                  <div className="mt-1 truncate text-xs text-red-400" title={f.error}>
                    {f.error}
                  </div>
                )}
              </div>
```

**분석 / Analysis (Level 2)**
- `min-w-0` + `truncate` 조합 — flex item에서 텍스트 잘림이 작동하게 하는 표준 트릭(min-w-0 없으면 truncate 무력화).
- 사이즈 표시가 항상 MB 단위. 작은 파일은 0.0 MB로 보일 수 있음.
- `stage.message` 가 백엔드에서 그대로 옴 → 신뢰 가능한 소스라면 OK, 신뢰 못 한다면 입력 검증 필요.

```tsx path=src/components/FileList.tsx lines=95-118
              <span
                className={cn(
                  "rounded-full px-2 py-0.5 text-[11px] font-medium",
                  statusStyle[f.status],
                )}
              >
                {statusLabel[f.status]}
              </span>

              <button
                className="invisible rounded p-1.5 text-fg-subtle hover:bg-bg hover:text-fg group-hover:visible"
                onClick={(e) => {
                  e.stopPropagation();
                  removeFile(f.id);
                }}
                title="파일 제거"
              >
                <IconTrash width={14} height={14} />
              </button>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
```

**분석 / Analysis (Level 2)**
- `invisible ... group-hover:visible` — hover 전에는 보이지 않는다. 키보드 사용자에게는 불친절(focus 시 visible 도 추가하면 좋다).
- `e.stopPropagation()` — 행 onClick(setActiveFile)이 함께 호출되지 않도록 막는다.
- 삭제 확인 다이얼로그 없이 즉시 삭제 → 사이드바의 프로젝트 삭제는 confirm을 거치는데, 파일은 미사용. 데이터 손실 위험은 낮지만 일관성 결여.

**짚어볼 점 / Watch-outs**
- 삭제 확인 부재.
- 접근성: 키보드 포커스 시에도 삭제 버튼이 안 보임.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - Exhaustive `Record<FileStatus, ...>` 룩업 테이블로 상태 누락이 컴파일에서 막힌다.
  - 정보(파일명/시간/크기/스테이지/오류) 위계가 명확.
- 위험 / Risks:
  - 파일 삭제 시 확인 다이얼로그 없음 (UX 일관성/실수 위험).
  - hover 전용 삭제 버튼 → 접근성 미흡.
  - `Stage` 타입 중복 정의 (DRY 위반).
- 개선 권고 / Recommendations:
  - `confirmDialog`(Sidebar.tsx)를 재사용해 파일 삭제도 확인 거치도록.
  - `group-hover:visible` 옆에 `focus-within:visible` 추가.
  - `Stage` 타입을 `usePipelineStore` 에서 export 해 공유.
- 재읽기 권고 레벨 / Suggested re-read level: 1.
