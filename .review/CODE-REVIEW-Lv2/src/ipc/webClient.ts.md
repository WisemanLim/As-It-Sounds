# src/ipc/webClient.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Web 모드 IPC 클라이언트 — REST(fetch) + SSE(EventSource)로 백엔드와 통신. tauriIpc 와 동일한 surface 노출.
- 진입점에서의 거리 / Distance from entrypoint: 3
- 라인 수 / LOC: 121
- 주요 export / Exports: `webIpc`, `webOnPipelineEvent`
- 의존 모듈 / Depends on (internal): `../types`, `../lib/env`
- 사용된 외부 패키지 / External packages used: fetch / EventSource (브라우저 내장)
- 부수효과 / Side effects: HTTP 요청, SSE 구독
- 동시성 모델 / Concurrency model: 각 fetch 는 독립 Promise. SSE 는 무한 스트림 + 브라우저 자동 재연결.
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/ipc/webClient.ts lines=1-22
// Web (browser) IPC client — talks to the axum server via REST + SSE.
// Surface matches `tauriIpc` so the rest of the app stays oblivious.

import type {
  MediaFile, NewFileInput, PipelineEvent, Project, Segment, Settings,
} from "../types";
import { apiBaseUrl } from "../lib/env";

function base(): string {
  return apiBaseUrl();
}

async function jget<T>(path: string): Promise<T> {
  const r = await fetch(`${base()}/api${path}`);
  if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
  return (await r.json()) as T;
}
```

**분석 / Analysis (Level 2)**
- `jget` 은 GET + JSON 응답. `await r.text().catch(() => r.statusText)` — 본문이 비어있거나 텍스트 추출 실패 시 statusText 로 폴백. 합리적 방어.
- 응답 타입은 `as T` 캐스팅 — 런타임 검증 없음. 백엔드가 잘못된 모양을 보내면 silent 통과.

```ts path=src/ipc/webClient.ts lines=24-39
async function jsend<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T | void> {
  const r = await fetch(`${base()}/api${path}`, {
    method,
    headers: body !== undefined ? { "content-type": "application/json" } : {},
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
  if (r.status === 204) return;
  const ct = r.headers.get("content-type") ?? "";
  if (!ct.includes("application/json")) return;
  return (await r.json()) as T;
}
```

**분석 / Analysis (Level 2)**
- 204 No Content 처리 — DELETE/PATCH 같은 응답을 안전하게 처리.
- content-type 가 json 이 아니면 본문 무시 → 비어있는 application/octet-stream 등의 응답 안전.
- 반환 타입이 `T | void` 이므로 호출 측에서 `as Promise<Settings>` 같은 추가 캐스팅 필요. 약간 어색하지만 동작은 안전.

**짚어볼 점 / Watch-outs**
- 네트워크 에러(타임아웃)에 별도 처리 없음 — fetch 가 throw 하므로 호출자가 catch 해야 함.
- CSRF 보호: 같은 origin 동일 cookie 면 표준 보호. 그러나 백엔드가 cross-origin 인 경우 별도 CSRF 토큰 미고려.

```ts path=src/ipc/webClient.ts lines=41-50
export const webIpc = {
  projectList: () => jget<Project[]>("/projects"),
  projectCreate: (name?: string) =>
    jsend<Project>("POST", "/projects", { name: name ?? null }) as Promise<Project>,
  projectRename: async (id: string, name: string): Promise<void> => {
    await jsend("PATCH", `/projects/${encodeURIComponent(id)}`, { name });
  },
  projectDelete: async (id: string): Promise<void> => {
    await jsend("DELETE", `/projects/${encodeURIComponent(id)}`);
  },
```

**분석 / Analysis (Level 2)**
- `encodeURIComponent(id)` — id 에 특수문자가 있어도 안전. URL 인젝션 방어 (CWE-89/CWE-79 약화).
- `projectCreate` 에서 `as Promise<Project>` 강제 캐스팅 — 204 path 가 없으므로 안전하지만 타입 시스템과 어긋남.

```ts path=src/ipc/webClient.ts lines=52-87
  fileList: (projectId: string) =>
    jget<MediaFile[]>(`/projects/${encodeURIComponent(projectId)}/files`),

  fileAddMany: async (
    _projectId: string,
    _inputs: NewFileInput[],
  ): Promise<MediaFile[]> => {
    throw new Error("fileAddMany is desktop-only — web uses fileUpload");
  },

  fileUpload: async (projectId: string, files: File[]): Promise<MediaFile[]> => {
    const form = new FormData();
    for (const f of files) form.append("files", f, f.name);
    const r = await fetch(
      `${base()}/api/projects/${encodeURIComponent(projectId)}/files`,
      { method: "POST", body: form },
    );
    if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
    return (await r.json()) as MediaFile[];
  },

  fileRemove: async (id: string): Promise<void> => {
    await jsend("DELETE", `/files/${encodeURIComponent(id)}`);
  },

  fileReset: async (id: string): Promise<void> => {
    await jsend("POST", `/files/${encodeURIComponent(id)}/reset`);
  },

  segmentsList: (fileId: string) =>
    jget<Segment[]>(`/files/${encodeURIComponent(fileId)}/segments`),
```

**분석 / Analysis (Level 2)**
- FormData multipart 업로드 — content-type 헤더를 fetch 가 자동으로 boundary 와 함께 설정. 헤더 수동 지정하면 깨지므로 명시 안 한 게 옳다.
- `f.name` 을 form.append 의 filename 으로 명시 — 일부 백엔드에서 filename 누락 시 처리가 곤란하므로 좋은 패턴.

**짚어볼 점 / Watch-outs**
- 업로드 진행률(progress)이 없음 — fetch API의 한계. XHR 또는 stream-based fetch 가 필요.
- 큰 파일을 메모리에 올려서 전송 — 매우 큰 미디어(수 GB)면 메모리 부담.

```ts path=src/ipc/webClient.ts lines=83-101
  settingsGet: () => jget<Settings>("/settings"),
  settingsSet: (settings: Settings) =>
    jsend<Settings>("PUT", "/settings", settings) as Promise<Settings>,

  pipelineRun: async (fileIds: string[]): Promise<void> => {
    await jsend("POST", "/pipeline/run", { fileIds });
  },

  translitPreview: async (text: string): Promise<string> => {
    const out = (await jsend<{ result: string }>("POST", "/translit/preview", {
      text,
    })) as { result: string };
    return out.result;
  },

  mediaSrc: (file: MediaFile): string =>
    `${base()}/api/files/${encodeURIComponent(file.id)}/media`,
};
```

**분석 / Analysis (Level 2)**
- `mediaSrc` 가 백엔드의 media endpoint를 직접 src 로 사용 — 인증 헤더가 필요 없는 공개 라우트 가정. 토큰 인증을 도입하려면 별도 처리 필요.
- `settingsSet` 으로 API 키 평문 전송 — HTTPS 라면 in-transit 안전. HTTP 라면 노출.

**짚어볼 점 / Watch-outs**
- Mixed content 위험: 페이지는 HTTPS, base URL이 HTTP 면 브라우저가 차단. 운영 환경 검증 필요.

```ts path=src/ipc/webClient.ts lines=103-121
export function webOnPipelineEvent(
  handler: (e: PipelineEvent) => void,
): Promise<() => void> {
  // EventSource auto-reconnects on transient drops; we expose a single sync
  // disposer for parity with Tauri's `UnlistenFn`.
  const es = new EventSource(`${base()}/api/events`);
  es.onmessage = (ev) => {
    try {
      handler(JSON.parse(ev.data) as PipelineEvent);
    } catch (e) {
      console.warn("bad SSE payload", e);
    }
  };
  es.onerror = () => {
    // Browser will auto-reconnect; surface only persistent failures via console.
    console.debug("SSE transient error");
  };
  return Promise.resolve(() => es.close());
}
```

**분석 / Analysis (Level 2)**
- EventSource 가 브라우저 표준으로 자동 재연결 처리.
- `JSON.parse` 실패에 대비한 try/catch — 손상된 SSE 페이로드를 견딘다.
- Promise.resolve 로 sync 함수를 Promise 인터페이스에 맞춤 — tauriOnPipelineEvent 와 시그니처 일치.

**짚어볼 점 / Watch-outs**
- handler 에서 throw 가 발생하면 onmessage 까지 전파되지 않지만, async handler 의 rejection 은 unhandledrejection 으로 가 fatal 오버레이를 트리거. PipelineListener 의 handler 는 동기이므로 OK.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - tauriIpc 와 정확히 일치하는 surface — 호출 측이 surface-agnostic.
  - URL 인코딩, 204 처리, content-type 분기 등 디테일 견고.
  - SSE 자동 재연결을 활용.
- 위험 / Risks:
  - 응답 타입 런타임 검증 없음 (silent drift 가능).
  - 큰 파일 업로드 시 메모리/진행률 미흡.
  - mediaSrc 가 인증 없이 직접 URL 사용 → 인증 도입 시 재설계 필요.
- 개선 권고 / Recommendations:
  - 응답 검증 (zod) 도입 또는 백엔드 OpenAPI codegen.
  - 업로드 진행률 노출을 원하면 XHR 또는 fetch stream 으로 전환.
  - SSE handler 에 fileId 별 시퀀스 검사 (out-of-order 방어).
- 재읽기 권고 레벨 / Suggested re-read level: 2.
