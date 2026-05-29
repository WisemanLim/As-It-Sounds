# src/components/PipelineListener.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 백엔드 파이프라인 이벤트를 받아 두 zustand store(`useAppStore`, `usePipelineStore`)에 반영하는 "invisible" 컴포넌트.
- 진입점에서의 거리 / Distance from entrypoint: 2 (App → here)
- 라인 수 / LOC: 60
- 주요 export / Exports: `PipelineListener`
- 의존 모듈 / Depends on (internal): `../lib/toast`, `../ipc/client`, `../store/usePipelineStore`, `../store/useAppStore`
- 사용된 외부 패키지 / External packages used: `react`
- 부수효과 / Side effects: IPC 이벤트 구독/해제, 두 store 업데이트, 토스트 표시
- 동시성 모델 / Concurrency model: `onPipelineEvent` Promise + 비동기 unlisten 등록
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/components/PipelineListener.tsx lines=1-5
import { useEffect } from "react";
import toast from "../lib/toast";
import { onPipelineEvent } from "../ipc/client";
import { usePipelineStore } from "../store/usePipelineStore";
import { useAppStore } from "../store/useAppStore";
```

**분석 / Analysis (Level 2)**
- 두 store + IPC client + toast 의 결합점. 명백한 의존성 응집이 보임.

```tsx path=src/components/PipelineListener.tsx lines=7-14
export function PipelineListener() {
  const setStage = usePipelineStore((s) => s.setStage);
  const setDownload = usePipelineStore((s) => s.setDownload);
  const setDone = usePipelineStore((s) => s.setDone);

  const patchFile = useAppStore((s) => s.patchFile);
  const upsertSegment = useAppStore((s) => s.upsertSegment);
  const clearSegmentsForFile = useAppStore((s) => s.clearSegmentsForFile);
```

**분석 / Analysis (Level 2)**
- 각 store 액션을 함수 참조로 분리 셀렉트. zustand 의 액션은 참조가 안정적이므로 deps 배열 사용이 안전.
- 액션을 객체 형태로 한 번에 뽑는 것보다 개별 select가 리렌더 횟수 면에서 유리.

```tsx path=src/components/PipelineListener.tsx lines=16-49
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    onPipelineEvent((e) => {
      switch (e.type) {
        case "fileStarted":
          patchFile(e.fileId, { status: "processing", error: undefined });
          clearSegmentsForFile(e.fileId);
          setStage(e.fileId, { stage: "decode", message: "시작..." });
          break;
        case "modelDownloading":
          setDownload({ progress: e.progress, message: e.message });
          break;
        case "progress":
          setStage(e.fileId, { stage: e.stage, message: e.message });
          break;
        case "segment":
          upsertSegment(e.segment);
          break;
        case "fileDone":
          patchFile(e.fileId, {
            status: "done",
            durationMs: e.durationMs ?? undefined,
          });
          setDone(e.fileId);
          break;
        case "fileError":
          patchFile(e.fileId, { status: "error", error: e.error });
          setDone(e.fileId);
          toast(`변환 실패: ${e.error}`);
          break;
        case "allDone":
          setDownload(null);
          break;
      }
    }).then((u) => {
      unlisten = u;
    });
```

**분석 / Analysis (Level 2)**
- discriminated union (`e.type`) 분기 → TS 가 각 case 의 페이로드를 자동 narrowing.
- `fileStarted` 에서 세그먼트를 비우고 다시 채워 retry/reset 경로에서 잔재가 남지 않게 함.
- `fileError` 시 toast로 즉시 사용자에게 알린다.
- `.then((u) => { unlisten = u; })` — Promise resolve 전에 컴포넌트 unmount 되면 `unlisten`이 아직 null 인 채로 cleanup이 끝나고, 이후 listener는 영원히 살아남는다.

**짚어볼 점 / Watch-outs**
- **레이스 컨디션 (CWE-362)**: subscribe Promise 가 resolve 되기 전 unmount → unlisten 핸들이 사라져 leak. StrictMode의 빠른 mount/unmount/mount 사이클이나 라우트 전환에서 재현 가능. `disposed` 플래그가 필요한 패턴(`useTauriDrop`은 이 패턴을 적용함).
- 의존성 배열에 모든 액션이 들어가 있어 zustand가 store 인스턴스를 새로 만들지 않는 한 effect는 정확히 한 번만 실행되지만, HMR/StrictMode 상황을 고려하면 추가 가드가 안전.

```tsx path=src/components/PipelineListener.tsx lines=53-59
    return () => {
      unlisten?.();
    };
  }, [setStage, setDownload, setDone, patchFile, upsertSegment, clearSegmentsForFile]);

  return null;
}
```

**분석 / Analysis (Level 2)**
- cleanup 에서 optional chaining 으로 안전 호출.
- 컴포넌트는 DOM을 렌더링하지 않는다. App 트리의 부수효과 전용 노드("invisible component") 패턴 — 라우트가 바뀌어도 한 번만 마운트되어 글로벌 리스너 역할.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 한 곳에 이벤트→상태 매핑이 모여 있어 흐름 추적이 쉽다.
  - 모든 PipelineEvent variant 를 명시적으로 처리.
- 위험 / Risks:
  - subscribe Promise resolve 전 unmount 시 listener leak (race condition).
  - `progress.stage` 가 자유 문자열이라 UI에서 그대로 노출되는 경로가 있음 — XSS는 React 가 처리하지만, 의미가 모호한 stage 값이 UI에 노출될 가능성.
- 개선 권고 / Recommendations:
  - `disposed` 플래그 + Promise resolve 시점 가드 (`useTauriDrop` 스타일):
    ```ts
    let disposed = false;
    onPipelineEvent(...).then(u => { if (disposed) u(); else unlisten = u; });
    return () => { disposed = true; unlisten?.(); };
    ```
  - 에러 토스트에 fileId 또는 파일명을 포함하면 다파일 시나리오에서 식별이 쉬워진다.
- 재읽기 권고 레벨 / Suggested re-read level: 2 — race-condition 패치 후 한 번 더 정독.
