# src/lib/cn.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Tailwind 클래스 합성 유틸. `clsx` + `tailwind-merge` 의 표준 콤보.
- 진입점에서의 거리 / Distance from entrypoint: 2 (대부분의 컴포넌트가 사용)
- 라인 수 / LOC: 6
- 주요 export / Exports: `cn`
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: `clsx`, `tailwind-merge`
- 부수효과 / Side effects: 없음 (순수 함수)
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/lib/cn.ts lines=1-6
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

**분석 / Analysis (Level 2)**
- `clsx` 가 truthy 값만 모아 공백 구분 문자열로 만들고, `twMerge` 가 Tailwind 의 충돌하는 유틸 클래스(`p-2`/`p-4` 등)에서 마지막 것만 남긴다.
- 거의 모든 shadcn/ui 스타일 프로젝트에서 동일한 6줄 — 안전한 표준.
- `ClassValue` 타입 재export로 호출 측이 추가 import 없이 사용 가능.

**짚어볼 점 / Watch-outs**
- `twMerge` 가 Tailwind 의 전체 클래스 사전을 내부에 갖고 있어 런타임 부하는 작지만, 매 렌더 호출 시 약간의 비용. 일반적인 React 앱에서는 무시 가능.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 표준 패턴 충실. 한 줄 함수로 의도 명확.
- 위험 / Risks: 없음.
- 개선 권고 / Recommendations: 없음.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
