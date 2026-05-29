# src/lib/filePicker.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 데스크톱(Tauri) / 웹 모두에서 사용자가 미디어 파일을 고르도록 하는 통합 picker.
- 진입점에서의 거리 / Distance from entrypoint: 2
- 라인 수 / LOC: 99
- 주요 export / Exports: `PickedMedia` (type), `pickMediaFiles`, `isAllowedMediaName`
- 의존 모듈 / Depends on (internal): `../types`, `../store/useAppStore` (detectKind), `./env`
- 사용된 외부 패키지 / External packages used: dynamic `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-fs`
- 부수효과 / Side effects: DOM input 생성/제거, native dialog 호출
- 동시성 모델 / Concurrency model: Promise 기반, browser fallback 에 `cancel` 이벤트 보조
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/lib/filePicker.ts lines=1-25
// File pickers — mode-aware. On desktop opens the Tauri dialog (returns absolute
// paths the Rust backend can read); on web opens a hidden `<input type="file">`
// (returns browser File objects that the caller uploads via multipart).

import type { NewFileInput } from "../types";
import { detectKind } from "../store/useAppStore";
import { isTauri } from "./env";

const EXTS = [
  "mp3", "wav", "m4a", "flac", "ogg",
  "mp4", "mov", "mkv", "webm", "avi", "m4v",
];

export type PickedMedia =
  | { mode: "paths"; inputs: NewFileInput[] }
  | { mode: "files"; files: File[] };
```

**분석 / Analysis (Level 2)**
- 헤더 주석이 디자인 의도(절대 경로 vs File 객체)를 명확히 설명.
- `EXTS` 가 `useTauriDrop.ts` 와 중복 정의 (drift 위험 — 두 번째 인식).
- discriminated union `PickedMedia` 가 desktop/web 페이로드를 한 타입으로 통합.

```ts path=src/lib/filePicker.ts lines=27-30
function baseName(p: string): string {
  const sep = p.includes("\\") ? "\\" : "/";
  return p.split(sep).pop() || p;
}
```

**분석 / Analysis (Level 2)**
- `useTauriDrop.ts` 와 정확히 동일한 코드 — DRY 위반 (세 번째 인식).
- 두 함수를 `lib/path.ts` 로 합치는 게 자연스러움.

```ts path=src/lib/filePicker.ts lines=32-57
async function pickViaTauri(): Promise<PickedMedia> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const { stat } = await import("@tauri-apps/plugin-fs");
  const res = await open({
    multiple: true,
    directory: false,
    filters: [{ name: "Audio & Video", extensions: EXTS }],
  });
  if (!res) return { mode: "paths", inputs: [] };
  const paths = Array.isArray(res) ? res : [res];
  const out: NewFileInput[] = [];
  for (const path of paths) {
    const p =
      typeof path === "string" ? path : (path as { path: string }).path;
    let size: number | undefined;
    try {
      const info = await stat(p);
      size = typeof info.size === "number" ? info.size : undefined;
    } catch {
      size = undefined;
    }
    const name = baseName(p);
    out.push({ name, path: p, kind: detectKind(name), sizeBytes: size });
  }
  return { mode: "paths", inputs: out };
}
```

**분석 / Analysis (Level 2)**
- Tauri 의 `open` 이 string 또는 `{path: string}` 객체를 반환할 수 있어 둘 다 처리. (Tauri 버전 호환)
- `multiple: true` 라도 사용자가 취소하면 `res` 가 null/undefined — 빈 inputs 로 폴백.
- 빈 catch — useTauriDrop 과 동일 (stat 실패 흡수).

```ts path=src/lib/filePicker.ts lines=59-89
function pickViaBrowser(): Promise<PickedMedia> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.multiple = true;
    input.accept = EXTS.map((e) => `.${e}`).join(",");
    // Some browsers garbage-collect detached inputs before `change` fires; keep
    // a reference until the picker resolves.
    document.body.appendChild(input);
    input.addEventListener(
      "change",
      () => {
        const files = input.files ? Array.from(input.files) : [];
        input.remove();
        resolve({ mode: "files", files });
      },
      { once: true },
    );
    // Edge case: user cancels — no `change` event fires. We get a `cancel`
    // event in modern browsers; fall back to a focus listener for the rest.
    input.addEventListener(
      "cancel",
      () => {
        input.remove();
        resolve({ mode: "files", files: [] });
      },
      { once: true },
    );
    input.click();
  });
}
```

**분석 / Analysis (Level 2)**
- DOM 에 input 을 append 한 이유가 코멘트로 명시 — 디테일한 호환성 처리.
- `{ once: true }` 옵션으로 자동 removeEventListener.
- `cancel` 이벤트는 모던 브라우저에서만 지원 → 구형 브라우저에서는 Promise 가 영원히 미해결 (메모리 누수 가능). 코멘트는 "focus listener" fallback 을 언급했지만 코드에는 구현되지 않음 → 부분 구현.

**짚어볼 점 / Watch-outs**
- 구형 브라우저(특히 Safari < 16)에서 사용자가 취소하면 Promise 미해결.
- 또한 사용자가 picker 를 열고 다른 페이지로 이동해도 listener 가 살아남는다 — 메모리 누수 위험 (소규모).

```ts path=src/lib/filePicker.ts lines=91-98
export function pickMediaFiles(): Promise<PickedMedia> {
  return isTauri ? pickViaTauri() : pickViaBrowser();
}

export function isAllowedMediaName(name: string): boolean {
  const ext = name.split(".").pop()?.toLowerCase();
  return !!ext && EXTS.includes(ext);
}
```

**분석 / Analysis (Level 2)**
- 단일 진입 함수 — 사용 측은 isTauri 를 알 필요 없다.
- `isAllowedMediaName` 가 DropZone 의 클라이언트측 필터링과 동일한 EXTS 를 사용 → 일관성 OK.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 데스크톱/웹 양쪽 picker 를 단일 API 로 통합.
  - 헤더/인라인 코멘트가 의도를 잘 설명.
  - 화이트리스트 확장자 검증.
- 위험 / Risks:
  - EXTS, baseName 가 `useTauriDrop.ts` 와 중복 정의 → drift 위험.
  - 구형 브라우저에서 picker 취소 시 Promise 미해결.
  - 코멘트가 약속한 "focus listener fallback" 미구현.
- 개선 권고 / Recommendations:
  - `lib/media.ts` 또는 `lib/path.ts` 로 EXTS/baseName/detectKind 통합.
  - `body` focus event 또는 `window.requestAnimationFrame` 폴링으로 cancel 미지원 브라우저 보강.
- 재읽기 권고 레벨 / Suggested re-read level: 1.
