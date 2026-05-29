# src/hooks/useTauriDrop.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Tauri 데스크톱 환경의 윈도우 레벨 파일 드롭을 받아 콜백으로 전달하는 훅. 웹에서는 no-op.
- 진입점에서의 거리 / Distance from entrypoint: 3 (HomePage / ProjectPage 가 사용)
- 라인 수 / LOC: 111
- 주요 export / Exports: `useTauriDrop`
- 의존 모듈 / Depends on (internal): `../types`, `../store/useAppStore` (detectKind), `../lib/env`, `../lib/filePicker`
- 사용된 외부 패키지 / External packages used: `react`, dynamic `@tauri-apps/plugin-fs`, `@tauri-apps/api/webview`
- 부수효과 / Side effects: 글로벌 webview 드래그/드롭 리스너 등록 및 해제
- 동시성 모델 / Concurrency model: async setup + `disposed` 플래그 + 비동기 unlisten 보호
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/hooks/useTauriDrop.ts lines=1-9
// Window-level file drop handler for the Tauri webview.
// On web this hook is a no-op — the browser `DropZone` element handles HTML5
// drag/drop directly there.

import { useEffect, useRef } from "react";
import type { NewFileInput } from "../types";
import { detectKind } from "../store/useAppStore";
import { isTauri } from "../lib/env";
import type { PickedMedia } from "../lib/filePicker";
```

**분석 / Analysis (Level 2)**
- 헤더 주석이 훅의 의도를 잘 설명. 하이브리드 surface (desktop/web) 분기의 한 축.
- `detectKind` 가 store 모듈에서 export 되는데 store 와는 무관한 순수 함수 → 위치가 어색. `lib/`에 두는 게 더 자연스러움.

```ts path=src/hooks/useTauriDrop.ts lines=11-28
const EXTS = new Set([
  "mp3", "wav", "m4a", "flac", "ogg",
  "mp4", "mov", "mkv", "webm", "avi", "m4v",
]);

function baseName(p: string): string {
  const sep = p.includes("\\") ? "\\" : "/";
  return p.split(sep).pop() || p;
}
```

**분석 / Analysis (Level 2)**
- `Set` 으로 빠른 O(1) 조회.
- EXTS 가 `filePicker.ts` 와 중복 정의 → DRY 위반. 확장자 추가 시 두 곳 모두 갱신 필요 (drift 위험).
- Windows 경로(`\`)와 POSIX 경로(`/`) 둘 다 처리하는 baseName — desktop 의 cross-platform 친화.

**짚어볼 점 / Watch-outs**
- 확장자 목록 drift. 한 곳(예: `lib/media.ts`)에서 export 해 import 하는 형태가 안전.

```ts path=src/hooks/useTauriDrop.ts lines=30-47
async function tauriPathsToInputs(paths: string[]): Promise<NewFileInput[]> {
  const { stat } = await import("@tauri-apps/plugin-fs");
  const out: NewFileInput[] = [];
  for (const p of paths) {
    const name = baseName(p);
    const ext = name.split(".").pop()?.toLowerCase();
    if (!ext || !EXTS.has(ext)) continue;
    let size: number | undefined;
    try {
      const info = await stat(p);
      size = typeof info.size === "number" ? info.size : undefined;
    } catch {
      size = undefined;
    }
    out.push({ name, path: p, kind: detectKind(name), sizeBytes: size });
  }
  return out;
}
```

**분석 / Analysis (Level 2)**
- 동적 import → 웹 번들에 Tauri 모듈이 포함되지 않게 한다.
- `stat` 실패 시 size 만 undefined 로 두고 파일은 유지 — 사이즈가 없어도 처리 가능.
- 화이트리스트 확장자 검증으로 잘못된 파일 유입 차단.

**짚어볼 점 / Watch-outs**
- 빈 catch 블록 — 무엇이 실패했는지 로깅이 없다 (CWE-703 swallowed exception 수준 낮음. 보조 정보 손실은 OK).

```ts path=src/hooks/useTauriDrop.ts lines=49-61
export function useTauriDrop(
  onDrop: (picked: PickedMedia) => void,
  onHover?: (hovering: boolean) => void,
) {
  const onDropRef = useRef(onDrop);
  const onHoverRef = useRef(onHover);

  useEffect(() => {
    onDropRef.current = onDrop;
  }, [onDrop]);
  useEffect(() => {
    onHoverRef.current = onHover;
  }, [onHover]);
```

**분석 / Analysis (Level 2)**
- "stable callback" 패턴 — 콜백을 ref 에 저장해 listener effect 가 다시 마운트되지 않게 한다. zustand 액션은 안정적이라 굳이 필요 없을 수도 있지만, 임의 prop 콜백을 받을 때는 좋은 방어.
- React 18 StrictMode 에서도 listener 의 중복 setup 을 피하는 데 도움.

```ts path=src/hooks/useTauriDrop.ts lines=63-110
  useEffect(() => {
    if (!isTauri) return;
    let disposed = false;
    let unlisten: (() => void) | null = null;
    const safeUnlisten = (fn: (() => void) | null) => {
      if (!fn) return;
      try {
        fn();
      } catch (e) {
        console.debug("drop listener already unregistered", e);
      }
    };
    const setup = async () => {
      try {
        const { getCurrentWebview } = await import("@tauri-apps/api/webview");
        const u = await getCurrentWebview().onDragDropEvent(async (event) => {
          if (disposed) return;
          const payload = event.payload as unknown as {
            type?: string;
            paths?: string[];
          };
          if (payload.type === "enter" || payload.type === "over") {
            onHoverRef.current?.(true);
          } else if (payload.type === "leave" || payload.type === "cancel") {
            onHoverRef.current?.(false);
          } else if (payload.type === "drop") {
            onHoverRef.current?.(false);
            const inputs = await tauriPathsToInputs(payload.paths ?? []);
            if (inputs.length > 0)
              onDropRef.current({ mode: "paths", inputs });
          }
        });
        if (disposed) {
          safeUnlisten(u);
          return;
        }
        unlisten = u;
      } catch (e) {
        console.warn("native drag/drop unavailable", e);
      }
    };
    setup();
    return () => {
      disposed = true;
      safeUnlisten(unlisten);
      unlisten = null;
    };
  }, []);
}
```

**분석 / Analysis (Level 2)**
- **모범적인 race-condition 처리**: subscribe Promise 가 resolve 된 시점에 `disposed` 가 true 면 즉시 unlisten — listener leak 방지.
- `safeUnlisten` 으로 unlisten 자체 실패도 흡수.
- `useEffect` deps 가 `[]` — 마운트 시 한 번만 실행. ref 패턴 덕에 콜백 변경 시 재마운트 불필요.
- payload 를 `as unknown as ...` 두 단계 캐스팅 — TS strictness 우회. 실제 Tauri 타입 정의를 가져왔다면 더 안전.

**짚어볼 점 / Watch-outs**
- `PipelineListener.tsx` 와 비교하면 여기는 race-condition 처리가 완벽하다 → 두 파일을 같은 패턴으로 통일하는 게 좋다.
- `disposed` 가 true 일 때 호버 콜백/드롭 콜백은 무시. 사용자에게는 이벤트 손실이 보일 수 있지만 unmount 직후이므로 무의미.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - race-condition 안전한 listener 등록/해제 (`disposed` + `safeUnlisten`).
  - ref 패턴으로 콜백 변경에도 effect 재실행 없음.
  - 동적 import 로 웹 번들 사이즈 절약.
- 위험 / Risks:
  - EXTS 가 `filePicker.ts` 와 중복 정의 → drift.
  - 빈 catch 가 stat 실패 사유를 삼킴 (low impact).
  - `detectKind` 가 store 모듈에서 import 되는 의존 방향이 어색.
- 개선 권고 / Recommendations:
  - EXTS / `detectKind` / `baseName` 을 공통 `lib/media.ts` 로 추출.
  - stat 실패 시 `console.debug("stat failed", p, e)` 로 보조 로깅.
  - PipelineListener 에 같은 race 패턴 적용.
- 재읽기 권고 레벨 / Suggested re-read level: 1.
