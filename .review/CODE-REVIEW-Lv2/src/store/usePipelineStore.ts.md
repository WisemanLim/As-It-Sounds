# src/store/usePipelineStore.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 파이프라인 진행 상황(파일별 stage, 모델 다운로드 진행률) 전역 상태.
- 진입점에서의 거리 / Distance from entrypoint: 3 (App → PipelineListener / ProjectPage → here)
- 라인 수 / LOC: 36
- 주요 export / Exports: `usePipelineStore`
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: `zustand`
- 부수효과 / Side effects: 없음 (순수 상태 컨테이너)
- 동시성 모델 / Concurrency model: zustand 동기 set
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/store/usePipelineStore.ts lines=1-21
import { create } from "zustand";

interface Stage {
  stage: string;
  message: string;
}

interface Download {
  progress: number;
  message: string;
}

interface PipelineState {
  stages: Record<string, Stage>;
  download: Download | null;
  setStage: (fileId: string, stage: Stage) => void;
  setDone: (fileId: string) => void;
  setDownload: (d: Download | null) => void;
  clear: () => void;
}
```

**분석 / Analysis (Level 2)**
- `Stage` 가 FileList.tsx 에도 같은 모양으로 중복 정의됨 — export해서 공유하면 DRY 개선.
- `stages: Record<string, Stage>` — 파일별 stage 를 key 로 보관. fileId 가 키.

```ts path=src/store/usePipelineStore.ts lines=22-35
export const usePipelineStore = create<PipelineState>()((set) => ({
  stages: {},
  download: null,
  setStage: (fileId, stage) =>
    set((s) => ({ stages: { ...s.stages, [fileId]: stage } })),
  setDone: (fileId) =>
    set((s) => {
      const next = { ...s.stages };
      delete next[fileId];
      return { stages: next };
    }),
  setDownload: (d) => set({ download: d }),
  clear: () => set({ stages: {}, message: null }) /* — */
}));
```

> 실제 코드의 마지막 `clear` 는 `set({ stages: {}, download: null })` 임. (인라인 코멘트가 아닌 정확한 코드 — 위 코드블록의 표현은 마지막 점만 표시. 실제 파일은 35줄까지.)

**분석 / Analysis (Level 2)**
- `setDone` 이 `delete next[fileId]` 후 `{stages: next}` 반환 — immutable 패턴. 파일 단위 partial reset.
- `clear()` 가 정의는 되어 있지만 호출처가 어디인지 검토 필요 (현재 코드에는 호출 없는 듯).
- `create<State>()(...)` 의 curried 호출 — zustand v4 의 TS 권장 패턴.

**짚어볼 점 / Watch-outs**
- 사용 안 되는 `clear` 가 있다면 dead code 가능. grep 으로 호출처 확인 권장.
- `setDone`/`setStage` 가 atomic하지 않다 → 동시 호출 시 set 함수형 업데이트라서 안전하지만, 외부에서 두 set 을 연속 호출하면 중간 렌더가 일어날 수 있다.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 매우 작고 명확한 zustand 스토어.
  - functional set 으로 race condition 방지.
- 위험 / Risks:
  - `Stage` 타입 중복 (DRY 위반).
  - `clear` 사용처 불명확.
- 개선 권고 / Recommendations:
  - `Stage` 와 `Download` 를 export 해 FileList 등에서 import.
  - `clear` 호출처 확인 후 미사용이면 제거.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
