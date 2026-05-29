# src/ipc/client.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: surface dispatcher — `isTauri` 플래그로 tauriIpc 또는 webIpc 를 선택해 단일 export 로 노출.
- 진입점에서의 거리 / Distance from entrypoint: 3
- 라인 수 / LOC: 16
- 주요 export / Exports: `ipc`, `onPipelineEvent`
- 의존 모듈 / Depends on (internal): `../types`, `../lib/env`, `./tauriClient`, `./webClient`
- 사용된 외부 패키지 / External packages used: 없음 (간접만)
- 부수효과 / Side effects: 없음
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/ipc/client.ts lines=1-9
// IPC switcher — picks the desktop (Tauri) or web (HTTP/SSE) client at runtime.
// Components import `ipc` and `onPipelineEvent` here without caring which
// surface they're running on.

import type { PipelineEvent } from "../types";
import { isTauri } from "../lib/env";
import { tauriIpc, tauriOnPipelineEvent } from "./tauriClient";
import { webIpc, webOnPipelineEvent } from "./webClient";
```

**분석 / Analysis (Level 2)**
- 두 클라이언트를 정적 import → 양쪽 코드 모두 번들에 포함됨. tree-shaking 효과 제한적이지만, 두 모듈 모두 작은 wrapper 라 부담 적음.
- Tauri 가 web 빌드에 들어와도 호출되지 않으면 부작용 없음 — 하지만 dynamic import 로 분리하면 web 번들에서 `@tauri-apps/*` 트리쉐이킹 가능.

**짚어볼 점 / Watch-outs**
- Vite 의 conditional compile 이 적용되지 않으면 web 빌드에 Tauri 모듈이 그대로 들어가 번들 사이즈 증가. 큰 문제는 아니나 최적화 여지.

```ts path=src/ipc/client.ts lines=10-16
export const ipc = isTauri ? tauriIpc : webIpc;

export function onPipelineEvent(
  handler: (e: PipelineEvent) => void,
): Promise<() => void> {
  return isTauri ? tauriOnPipelineEvent(handler) : webOnPipelineEvent(handler);
}
```

**분석 / Analysis (Level 2)**
- 단일 진입 — 호출 측은 `import { ipc, onPipelineEvent } from "../ipc/client"` 만 알면 된다. 깔끔한 facade 패턴.
- `tauriOnPipelineEvent` 는 `Promise<UnlistenFn>`, `webOnPipelineEvent` 는 `Promise<() => void>` — TS 가 두 타입을 모두 `Promise<() => void>` 로 받아들이는지 확인. UnlistenFn 이 `() => void` 별칭이라면 OK.
- 호출 시점에 매번 `isTauri` 분기 — 작은 비용. 모듈 로드 시 한 번만 분기하는 `ipc` 와 동일 패턴으로 했으면 일관성 향상.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 매우 단순한 facade — 호출 측 코드가 surface-agnostic.
  - 의도 주석이 명확.
- 위험 / Risks:
  - web 번들에 Tauri 모듈이 들어가는 사이즈 패널티 (소규모).
  - onPipelineEvent 가 매 호출마다 isTauri 분기 — 작은 비용/일관성 결여.
- 개선 권고 / Recommendations:
  - `export const onPipelineEvent = isTauri ? tauriOnPipelineEvent : webOnPipelineEvent;` 로 단순화.
  - Vite의 `define`/`build.rollupOptions.external` 등으로 Tauri 모듈 web 제외.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
