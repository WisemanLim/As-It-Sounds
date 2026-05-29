# src/lib/env.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 런타임 Tauri/web 환경 감지 + API base URL 헬퍼.
- 진입점에서의 거리 / Distance from entrypoint: 2
- 라인 수 / LOC: 21
- 주요 export / Exports: `isTauri`, `apiBaseUrl`
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: Vite `import.meta.env`
- 부수효과 / Side effects: 없음 (`isTauri` 는 모듈 로드 시 한 번 평가)
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/lib/env.ts lines=1-12
// Runtime detection for Tauri (desktop) vs browser (web).
// The same React bundle ships to both surfaces; ipc / file pickers / drop
// hooks switch on this flag at module load.

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
```

**분석 / Analysis (Level 2)**
- `__TAURI_INTERNALS__` 글로벌은 Tauri 가 webview 부팅 시 주입하는 비공식 마커. 공식 `window.__TAURI__` 도 있지만 v2 에서 internal flag 가 더 빠르게 들어감.
- 모듈 로드 타임에 평가되므로 SSR / 테스트 환경에서 window 가 없을 때 안전 (`typeof window !== "undefined"`).
- `unknown` 으로 타입 지정 — 어떤 값이든 받을 수 있도록.

**짚어볼 점 / Watch-outs**
- `__TAURI_INTERNALS__` 가 비공식 → Tauri 메이저 버전 업그레이드 시 깨질 가능성. 공식 `__TAURI__` 도 함께 체크하면 더 견고.

```ts path=src/lib/env.ts lines=14-21
/**
 * Base URL for the web API server. Reads `VITE_AIS_API_BASE` at build time
 * (typically left empty so the frontend hits the same origin behind nginx).
 */
export function apiBaseUrl(): string {
  const env = (import.meta as unknown as { env?: Record<string, string> }).env;
  return env?.VITE_AIS_API_BASE ?? "";
}
```

**분석 / Analysis (Level 2)**
- Vite 가 빌드 시 `import.meta.env.VITE_AIS_API_BASE` 를 정적 치환.
- `as unknown as { env? ... }` 두 단계 캐스팅 — TS strict 모드에서 `import.meta.env` 타입을 직접 모르는 경우의 우회. 본래는 `vite-env.d.ts` 에 `ImportMetaEnv` 인터페이스를 추가하면 깔끔.
- 빈 문자열 폴백 → 동일 origin (`/api/...`) 라우팅을 위한 의도.

**짚어볼 점 / Watch-outs**
- `vite-env.d.ts` 에 `ImportMetaEnv`/`ImportMeta` 타입 보강하지 않은 점 → 타입 단언 두 번이 들어가 보기 흉.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 모듈 로드 1회 평가로 분기 비용 0.
  - SSR/테스트 안전 가드.
- 위험 / Risks:
  - 비공식 `__TAURI_INTERNALS__` 의존 → 버전 호환 리스크.
  - `import.meta.env` 타입 보강 미흡.
- 개선 권고 / Recommendations:
  - `__TAURI__` 와 OR 로 ​함께 체크.
  - `vite-env.d.ts` 에 `interface ImportMetaEnv { VITE_AIS_API_BASE: string }` 정의.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
