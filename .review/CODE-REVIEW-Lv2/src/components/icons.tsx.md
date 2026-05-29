# src/components/icons.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 외부 아이콘 라이브러리 의존성을 없애기 위한 인라인 SVG 아이콘 모음 (Feather-style).
- 진입점에서의 거리 / Distance from entrypoint: 2 (여러 컴포넌트가 사용)
- 라인 수 / LOC: 110
- 주요 export / Exports: `IconMenu`, `IconPlus`, `IconSearch`, `IconSettings`, `IconUpload`, `IconTrash`, `IconPlay`, `IconMic`, `IconFile`, `IconVideo`, `IconAudio`, `IconChevronRight`, `IconSparkles`
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: `react` (타입 `SVGProps`)
- 부수효과 / Side effects: 없음 (순수 컴포넌트)
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/components/icons.tsx lines=1-14
import type { SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement>;

const base = {
  width: 20,
  height: 20,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.8,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
};
```

**분석 / Analysis (Level 2)**
- `SVGProps<SVGSVGElement>` 로 모든 표준 SVG 속성 + ref/className/style 지원.
- `as const` 로 string literal narrowing — TS 가 string 이 아니라 `"round"` 로 추론하도록.
- `stroke="currentColor"` 가 핵심 — 부모 텍스트 색을 그대로 사용 (Tailwind `text-brand` 등으로 색 변경 가능).

**짚어볼 점 / Watch-outs**
- `base` spread 가 `{...p}` 보다 먼저 와서, 호출 측에서 `width`/`stroke` 등을 덮어쓸 수 있다. 의도된 디자인.

```tsx path=src/components/icons.tsx lines=16-43
export const IconMenu = (p: IconProps) => (
  <svg {...base} {...p}>
    <line x1="4" y1="6" x2="20" y2="6" />
    ...
  </svg>
);

export const IconPlus = (p: IconProps) => (
  <svg {...base} {...p}>
    <line x1="12" y1="5" x2="12" y2="19" />
    <line x1="5" y1="12" x2="19" y2="12" />
  </svg>
);

export const IconSearch = (p: IconProps) => (
  <svg {...base} {...p}>
    <circle cx="11" cy="11" r="7" />
    <line x1="21" y1="21" x2="16.65" y2="16.65" />
  </svg>
);

export const IconSettings = (p: IconProps) => (
  <svg {...base} {...p}>
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9 ... 1.7 1.7 0 0 0-1.5-1z" />
  </svg>
);
```

**분석 / Analysis (Level 2)**
- 각 아이콘이 `({...base, ...p})` 패턴 — base 의 기본값 위에 prop으로 override.
- viewBox/stroke 등 공통 속성이 base 에서 재사용되어 보일러플레이트 감소.

```tsx path=src/components/icons.tsx lines=45-97
export const IconUpload = (p: IconProps) => ( /* ... */ );
export const IconTrash = (p: IconProps) => ( /* ... */ );
export const IconPlay = (p: IconProps) => (
  <svg {...base} {...p}>
    <polygon points="6 3 21 12 6 21 6 3" fill="currentColor" />
  </svg>
);
export const IconMic = (p: IconProps) => ( /* ... */ );
export const IconFile = (p: IconProps) => ( /* ... */ );
export const IconVideo = (p: IconProps) => (
  <svg {...base} {...p}>
    <rect x="3" y="6" width="13" height="12" rx="2" />
    <polygon points="23 7 16 12 23 17 23 7" fill="currentColor" />
  </svg>
);
export const IconAudio = (p: IconProps) => ( /* ... */ );
```

**분석 / Analysis (Level 2)**
- `IconPlay`/`IconVideo`는 `fill="currentColor"`를 명시적으로 추가해 line이 아닌 채워진 도형으로 그려진다 — base의 `fill="none"`을 overrride.
- 아이콘들은 24x24 viewBox 표준 — Feather/Lucide와 동일한 그리드.

```tsx path=src/components/icons.tsx lines=99-110
export const IconChevronRight = (p: IconProps) => (
  <svg {...base} {...p}>
    <polyline points="9 18 15 12 9 6" />
  </svg>
);

export const IconSparkles = (p: IconProps) => (
  <svg {...base} {...p}>
    <path d="M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9L12 3z" />
    <path d="M19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8L19 16z" />
  </svg>
);
```

**분석 / Analysis (Level 2)**
- Sparkles는 별 모양 path 2개로 구성된 장식용. App 의 브랜드 아이콘.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 외부 아이콘 라이브러리 의존성 0 → 번들 사이즈 절약.
  - `stroke="currentColor"` 로 Tailwind 텍스트 색과 자연 통합.
- 위험 / Risks:
  - 거의 없음. 정적 SVG에 보안/성능 이슈가 발생하기 어려운 영역.
- 개선 권고 / Recommendations:
  - 아이콘 수가 늘면 컴포넌트 함수 양산보다 `<Icon name="..." />` 라우터 패턴이나 sprite 시트 고려.
  - `aria-hidden="true"` 또는 `role="img" aria-label` 정책을 명시하면 접근성 일관성 향상.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
