# src/mock/fixtures.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 데모/스토리북용 더미 세그먼트 데이터 생성기.
- 진입점에서의 거리 / Distance from entrypoint: 2+ (어떤 페이지에서도 직접 import 되지 않음 — 사용처 없는 상태로 보임)
- 라인 수 / LOC: 45
- 주요 export / Exports: `mockSegmentsForFile`
- 의존 모듈 / Depends on (internal): `../types`
- 사용된 외부 패키지 / External packages used: 없음
- 부수효과 / Side effects: 없음 (순수 함수)
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/mock/fixtures.ts lines=1-3
import type { Segment } from "../types";

export function mockSegmentsForFile(fileId: string): Segment[] {
```

**분석 / Analysis (Level 2)**
- `import type`로 타입만 가져옴 — 런타임 임포트가 아니어서 트리쉐이킹에 안전.
- `fileId`를 받아 그 파일에 속한 더미 세그먼트 배열을 만든다. 함수형 팩토리 패턴.

```ts path=src/mock/fixtures.ts lines=4-33
  const base = [
    {
      startMs: 0,
      endMs: 3120,
      en: "Hello everyone, welcome to As It Sounds.",
      han: "헬로 에브리원, 웰컴 투 애즈 잇 사운즈.",
      ko: "안녕하세요 여러분, As It Sounds에 오신 것을 환영합니다.",
    },
    {
      startMs: 3120,
      endMs: 6480,
      en: "This app converts English audio into Korean phonetic spelling.",
      han: "디스 앱 컨버츠 잉글리시 오디오 인투 코리안 포네틱 스펠링.",
      ko: "이 앱은 영어 오디오를 한국어 발음 표기로 변환합니다.",
    },
    {
      startMs: 6480,
      endMs: 10240,
      en: "You can also see the translation right below it.",
      han: "유 캔 올소 시 더 트랜슬레이션 라잇 빌로우 잇.",
      ko: "또한 바로 아래에서 번역을 볼 수 있습니다.",
    },
    {
      startMs: 10240,
      endMs: 13880,
      en: "Drag and drop your audio or video files to get started.",
      han: "드래그 앤 드롭 유어 오디오 오어 비디오 파일즈 투 겟 스타티드.",
      ko: "오디오나 영상 파일을 끌어다 놓으면 시작됩니다.",
    },
  ];
```

**분석 / Analysis (Level 2)**
- 4개의 짧은 데모 세그먼트. 영어 → 한글 음차 → 한국어 번역 3-line 구조를 그대로 보여준다.
- 인라인 한국어 텍스트가 그대로 들어있어 i18n에서 분리되어야 할 수 있지만, "데모용" 한정이라면 OK.

**짚어볼 점 / Watch-outs**
- 이 파일은 어떤 import에서도 실제로 사용되지 않는 것으로 보임 → dead code 가능성. `grep mockSegmentsForFile`로 확인 필요.

```ts path=src/mock/fixtures.ts lines=34-44
  return base.map((b, i) => ({
    id: `${fileId}-seg-${i}`,
    fileId,
    index: i,
    startMs: b.startMs,
    endMs: b.endMs,
    textEn: b.en,
    textHangul: b.han,
    textKo: b.ko,
  }));
}
```

**분석 / Analysis (Level 2)**
- `base` 의 키 이름(`en`, `han`, `ko`)을 `Segment` 의 키 이름(`textEn`, `textHangul`, `textKo`) 으로 매핑한다. 약간의 중복이지만 도메인 타입과의 매핑이 명시적이다.
- `id`는 `${fileId}-seg-${i}` 패턴 — 결정적이라 React key로 안정적.

**짚어볼 점 / Watch-outs**
- `base` 자체를 `Omit<Segment, 'id' | 'fileId' | 'index'>` 으로 만들었다면 매핑이 더 짧았겠지만, 명시성이 더 우선이라면 그대로도 OK.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 순수 함수, 결정적 id 생성, 도메인 타입과 깔끔하게 분리.
- 위험 / Risks:
  - 어디서도 사용되지 않으면 dead code (번들에 포함될 가능성은 적지만 유지 비용 발생).
- 개선 권고 / Recommendations:
  - 사용처 확인 후 dev 전용으로 격리 (`if (import.meta.env.DEV)` 가드 또는 Storybook 전용 폴더로 이동).
  - `base` 항목 타입을 명시 (`Pick<Segment, 'startMs' | 'endMs'> & { en; han; ko }`).
- 재읽기 권고 레벨 / Suggested re-read level: 0 — 짧고 위험도 낮다.
