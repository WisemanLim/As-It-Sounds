# src/components/SegmentView.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 세그먼트(영어/한글음차/한국어번역) 리스트 + 현재 재생 위치 하이라이트 + 클릭 시 시크.
- 진입점에서의 거리 / Distance from entrypoint: 3 (App → ProjectPage → here)
- 라인 수 / LOC: 73
- 주요 export / Exports: `SegmentView`
- 의존 모듈 / Depends on (internal): `../lib/cn`, `../lib/format`, `../types`
- 사용된 외부 패키지 / External packages used: 없음 직접 사용
- 부수효과 / Side effects: 없음 (`onSeek` 콜백)
- 동시성 모델 / Concurrency model: 동기 렌더 + 콜백
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/components/SegmentView.tsx lines=1-10
import { cn } from "../lib/cn";
import { formatTime } from "../lib/format";
import type { Segment } from "../types";

interface Props {
  segments: Segment[];
  currentTimeMs: number;
  onSeek?: (ms: number) => void;
}
```

**분석 / Analysis (Level 2)**
- props 가 명확 — segments 와 현재 시간만 받고 콜백은 옵셔널.
- 컴포넌트는 stateless presentational — 사이드 효과 없이 props 만 사용.

```tsx path=src/components/SegmentView.tsx lines=11-18
export function SegmentView({ segments, currentTimeMs, onSeek }: Props) {
  if (segments.length === 0) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-fg-muted">
        아직 변환된 세그먼트가 없습니다. "변환 시작"을 눌러 시작하세요.
      </div>
    );
  }
```

**분석 / Analysis (Level 2)**
- empty state — 안내 문구가 명확.
- 부모(ProjectPage) 에서 `segments` 길이 분기가 없으므로 여기서 처리하는 게 책임상 OK.

```tsx path=src/components/SegmentView.tsx lines=20-42
  return (
    <ol className="flex flex-col gap-2">
      {segments.map((seg) => {
        const active =
          currentTimeMs >= seg.startMs && currentTimeMs < seg.endMs;
        return (
          <li key={seg.id}>
            <button
              onClick={() => onSeek?.(seg.startMs)}
              className={cn(
                "w-full rounded-xl border px-4 py-3 text-left transition-colors",
                active
                  ? "border-brand/60 bg-brand-soft/30"
                  : "border-line bg-bg-elevated hover:bg-bg-hover",
              )}
            >
              <div className="flex items-center justify-between text-[11px] font-mono text-fg-subtle">
                <span>
                  {formatTime(seg.startMs)} — {formatTime(seg.endMs)}
                </span>
                <span>#{seg.index + 1}</span>
              </div>
```

**분석 / Analysis (Level 2)**
- `<ol>` + `<li>` 사용 — 순서가 의미 있는 세그먼트 리스트에 적절한 시맨틱.
- `<button>` 이 행 전체이므로 키보드/스크린리더 접근성 자동 확보 — 다른 컴포넌트(FileList, DropZone) 와 대비된다.
- `active` 계산이 매 렌더마다 모든 세그먼트에 대해 일어남 — segments 수가 매우 크면 O(N)/렌더지만 일반 음성 파일에선 무시 가능.

**짚어볼 점 / Watch-outs**
- 현재 활성 세그먼트로 자동 스크롤(scrollIntoView) 기능 없음 → 긴 리스트에서 사용자가 따라가기 어려울 수 있다.

```tsx path=src/components/SegmentView.tsx lines=43-66
              <div className="mt-2 space-y-1.5">
                <p className="text-sm leading-relaxed text-fg">
                  <span className="mr-2 inline-block rounded bg-bg-hover px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-fg-subtle">
                    EN
                  </span>
                  {seg.textEn}
                </p>
                {seg.textHangul && (
                  <p className="text-sm leading-relaxed text-brand">
                    <span className="mr-2 inline-block rounded bg-brand-soft px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-brand">
                      발음
                    </span>
                    {seg.textHangul}
                  </p>
                )}
                {seg.textKo && (
                  <p className="text-sm leading-relaxed text-fg-muted">
                    <span className="mr-2 inline-block rounded bg-bg-hover px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-fg-subtle">
                      번역
                    </span>
                    {seg.textKo}
                  </p>
                )}
              </div>
            </button>
          </li>
        );
      })}
    </ol>
  );
}
```

**분석 / Analysis (Level 2)**
- EN/발음/번역 3-line 형식이 명확하게 시각적으로 구분됨.
- React 가 문자열을 텍스트 노드로 그리므로 XSS 안전 (segment text 는 신뢰 가능한 백엔드에서 옴, 사용자 입력은 아님).

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 시맨틱 마크업(`<ol>/<li>/<button>`) + 키보드 접근성 우수.
  - 빈 상태/활성 상태 표현 명확.
- 위험 / Risks:
  - 활성 세그먼트로 자동 스크롤이 없어 긴 트랜스크립트 UX 가 약함.
  - 세그먼트가 수천 개를 넘으면 모든 활성 계산이 매 렌더 비용을 만든다 (현실적 데이터에서는 OK).
- 개선 권고 / Recommendations:
  - `useEffect` + `ref.scrollIntoView({block: 'nearest'})` 로 활성 세그먼트 자동 스크롤.
  - `Stage`/`Active` 계산을 부모에서 한 번만 해 prop으로 넘기는 최적화는 큰 데이터 셋 등장 시 고려.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
