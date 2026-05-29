# src/main.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 브라우저 진입점. React root 마운트, HashRouter 설정, 글로벌 fatal-error 오버레이.
- 진입점에서의 거리 / Distance from entrypoint: 0 (entry)
- 라인 수 / LOC: 55
- 주요 export / Exports: 없음 (top-level side-effects only)
- 의존 모듈 / Depends on (internal): `./App`, `./index.css`
- 사용된 외부 패키지 / External packages used: `react`, `react-dom/client`, `react-router-dom`
- 부수효과 / Side effects: `document.body.appendChild`, `window.addEventListener("unhandledrejection")`, `ReactDOM.createRoot().render(...)`
- 동시성 모델 / Concurrency model: 단일 동기 부팅 + 글로벌 promise rejection 핸들러
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/main.tsx lines=1-11
import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter } from "react-router-dom";
import App from "./App";
import "./index.css";

declare global {
  interface Window {
    __fatalShown?: boolean;
  }
}
```

**분석 / Analysis (Level 2)**
- 표준 React 18 진입(`react-dom/client`) + Tailwind 글로벌 CSS.
- `HashRouter` 선택은 Tauri 데스크톱(`tauri://`/`asset://`) 및 정적 호스팅과의 호환을 위해서다 — BrowserRouter는 file:// 스킴에서 동작이 까다롭다.
- `declare global` 로 `Window.__fatalShown` 추가 — fatal 오버레이 중복 방지용 플래그를 TS가 알게 해준다.

**짚어볼 점 / Watch-outs**
- 데스크톱이 아닌 웹 빌드에서도 HashRouter가 강제된다 → URL이 `/#/projects/:id` 형태. SEO나 share-link 측면에선 단점이지만 내부 도구이므로 OK.

```tsx path=src/main.tsx lines=13-34
function showFatal(message: string, stack?: string) {
  if (window.__fatalShown) return;
  window.__fatalShown = true;
  const overlay = document.createElement("div");
  overlay.style.cssText =
    "position:fixed;inset:0;z-index:2147483647;background:#0e0f11;color:#e6e7ea;" +
    "font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif;" +
    "padding:32px;display:flex;flex-direction:column;gap:12px;" +
    "align-items:center;justify-content:center;text-align:center;";
  const title = document.createElement("div");
  title.textContent = "오류가 발생했습니다";
  title.style.cssText = "color:#fca5a5;font-weight:600;";
  const pre = document.createElement("pre");
  pre.textContent = `${message}\n\n${stack ?? ""}`;
  pre.style.cssText =
    "max-width:720px;white-space:pre-wrap;word-break:break-word;" +
    "background:#17191c;padding:16px;border-radius:8px;" +
    "font-family:ui-monospace,Menlo,monospace;font-size:12px;color:#9aa0a6;text-align:left;";
  overlay.appendChild(title);
  overlay.appendChild(pre);
  document.body.appendChild(overlay);
}
```

**분석 / Analysis (Level 2)**
- Tailwind/React가 죽어도 보이는 "최후의 안전망" — 순수 DOM API만 사용해 오버레이를 그린다. 이게 main.tsx의 핵심 가치.
- `z-index:2147483647` (32-bit signed int max) → 다른 UI 위에 무조건 노출.
- `pre.textContent`로 stack을 넣는다 — `innerHTML`이 아니므로 XSS 안전 (Standing security check: injection ✅).
- `__fatalShown` 플래그로 중복 오버레이를 막는다.

**짚어볼 점 / Watch-outs**
- 인라인 스타일을 문자열 cssText로 직접 쓴다 → 유지보수성은 떨어지지만, CSS 파일이 망가져도 동작하게 하려는 의도이므로 합리적.
- 색상/폰트가 다크 테마 고정 — 라이트 모드 사용자에겐 어색할 수 있으나 fallback이므로 허용.

```tsx path=src/main.tsx lines=36-39
window.addEventListener("unhandledrejection", (e) => {
  const reason = e.reason as { message?: string; stack?: string } | undefined;
  showFatal(reason?.message ?? String(reason), reason?.stack);
});
```

**분석 / Analysis (Level 2)**
- 처리되지 않은 Promise rejection (예: `await` 없는 async 호출 실패)을 잡아 fatal 오버레이로 표시.
- React의 ErrorBoundary는 동기 렌더 에러만 잡으므로 async 에러는 이 핸들러가 보충한다.
- 타입 단언 `as { message?: string; stack?: string }` — `e.reason`은 사실상 `any` 이므로 narrow한 모양으로 캐스팅.

**짚어볼 점 / Watch-outs**
- 정상 흐름의 토스트 가능한 에러(예: 네트워크 실패)까지 fatal 오버레이로 잡힐 수 있다 → UX 폭주 위험. 현재 코드는 `catch` 잘 거는 컨벤션에 의존.
- "error" (synchronous) 이벤트는 잡지 않는다. 추가하면 더 견고.

```tsx path=src/main.tsx lines=41-55
try {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <HashRouter>
        <App />
      </HashRouter>
    </React.StrictMode>,
  );
} catch (e) {
  showFatal(
    e instanceof Error ? e.message : String(e),
    e instanceof Error ? e.stack : undefined,
  );
}
```

**분석 / Analysis (Level 2)**
- `getElementById("root") as HTMLElement` — index.html에 `<div id="root"></div>` 가 있다고 가정한 강제 캐스팅. 없으면 null인데 캐스팅으로 가려진다.
- StrictMode → dev에서 useEffect 두 번 실행 등 부작용 노출 도구. 프로덕션엔 영향 없음.
- 동기적인 createRoot/render 자체가 던지는 케이스(예: root null)도 fatal 오버레이로 잡힌다.

**짚어볼 점 / Watch-outs**
- `getElementById("root")` 가 null 이면 `createRoot(null)` 에서 TypeError. → fatal 오버레이로 잡히긴 하지만, 명시적으로 `if (!el) throw new Error(...)` 로 의도를 드러내면 더 좋다.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - React/Tailwind가 죽어도 보이는 fail-safe 오버레이 — fatal-shown 가드까지 들어 있어 견고.
  - unhandledrejection + try/catch + ErrorBoundary 의 3중 안전망.
- 위험 / Risks:
  - unhandledrejection이 너무 광범위해 사소한 비동기 실패까지 풀스크린 오버레이로 잡힐 수 있다.
  - `root` 엘리먼트 null 케이스가 명시적이지 않다.
- 개선 권고 / Recommendations:
  - "error" 글로벌 이벤트도 잡아 동기 throw 도 보호.
  - root null 가드 후 throw — 가독성 향상.
- 재읽기 권고 레벨 / Suggested re-read level: 1 — fail-safe 오버레이의 동작을 한 번 더 확인하면 충분.
