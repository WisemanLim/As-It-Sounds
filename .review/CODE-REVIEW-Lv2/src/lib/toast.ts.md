# src/lib/toast.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 최소 토스트 알림 유틸 — 화면 하단에 4초 동안 메시지 표시.
- 진입점에서의 거리 / Distance from entrypoint: 2
- 라인 수 / LOC: 15
- 주요 export / Exports: `default toast`
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: 없음
- 부수효과 / Side effects: DOM 노드 생성/제거, console.warn
- 동시성 모델 / Concurrency model: setTimeout 기반 자동 제거
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/lib/toast.ts lines=1-14
export default function toast(message: string) {
  if (typeof window === "undefined") return;
  console.warn("[toast]", message);
  try {
    const el = document.createElement("div");
    el.textContent = message;
    el.className =
      "fixed bottom-6 left-1/2 -translate-x-1/2 rounded-xl bg-red-900/80 px-4 py-2 text-sm text-red-100 shadow-lg backdrop-blur z-50";
    document.body.appendChild(el);
    setTimeout(() => el.remove(), 4000);
  } catch {
    // ignore DOM errors
  }
}
```

**분석 / Analysis (Level 2)**
- SSR 환경 가드(`typeof window === "undefined"`) — Vite 빌드는 SSR 이 아니지만 방어적.
- `textContent` 사용 → XSS 안전 (innerHTML 이 아님).
- 메시지 색상이 빨간색(`bg-red-900/80`) 으로 고정 — 모든 토스트가 에러 톤. 성공/경고 분기 없음.
- `console.warn` 으로 logs 에도 남김 → 디버깅에 유용.
- 동시 다발 토스트 시 위치가 겹친다 — `bottom-6` 이 절대치이므로 두 토스트가 같은 자리.

**짚어볼 점 / Watch-outs**
- 모든 토스트가 빨간색 → 일반 정보 알림에 부적절.
- 동시 다발 호출 시 시각적 겹침.
- 빈 catch — DOM 에러는 거의 없지만 swallowing OK.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 외부 토스트 라이브러리 없이 15 줄로 해결.
  - XSS 안전, SSR 안전.
- 위험 / Risks:
  - 단일 색상/스타일 → UX 다양성 부족.
  - 다중 토스트 겹침.
- 개선 권고 / Recommendations:
  - `toast.error`/`toast.info`/`toast.success` 같이 variant 분기.
  - 스택 컨테이너(`#toast-stack`)에 append 후 flex column 으로 정렬.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
