# src/ipc/tauriClient.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: Tauri 데스크톱 IPC 어댑터 — Rust 측 `invoke` 명령과 `listen` 이벤트의 타입 안전 래퍼.
- 진입점에서의 거리 / Distance from entrypoint: 3 (App → store/components → ipc/client → here)
- 라인 수 / LOC: 54
- 주요 export / Exports: `tauriIpc`, `tauriOnPipelineEvent`
- 의존 모듈 / Depends on (internal): `../types`
- 사용된 외부 패키지 / External packages used: `@tauri-apps/api/core`, `@tauri-apps/api/event`
- 부수효과 / Side effects: invoke/listen 호출 (Rust 측 명령 트리거)
- 동시성 모델 / Concurrency model: 각 invoke 는 독립 Promise. listen 은 무한 스트림.
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/ipc/tauriClient.ts lines=1-13
// Tauri (desktop) IPC client — talks to Rust via `invoke` and `listen`.

import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  MediaFile, Project, Segment, Settings, PipelineEvent, NewFileInput,
} from "../types";
```

**분석 / Analysis (Level 2)**
- `convertFileSrc` — 절대 경로를 `asset://localhost/...` 같이 webview 가 로드 가능한 URL 로 변환. 이걸 video/audio 의 src 에 그대로 사용한다.
- 정적 import — Tauri 전용 모듈은 빌드 분리 (Vite의 conditional compilation 또는 별도 entry) 가 안 되면 web 빌드에도 포함됨. `tauriClient.ts` 가 web 에서 import 되긴 하지만 함수 호출이 안 되므로 부트 자체는 OK.

```ts path=src/ipc/tauriClient.ts lines=14-30
export const tauriIpc = {
  projectList: () => invoke<Project[]>("project_list"),
  projectCreate: (name?: string) =>
    invoke<Project>("project_create", { name: name ?? null }),
  projectRename: (id: string, name: string) =>
    invoke<void>("project_rename", { id, name }),
  projectDelete: (id: string) => invoke<void>("project_delete", { id }),

  fileList: (projectId: string) =>
    invoke<MediaFile[]>("file_list", { projectId }),
  fileAddMany: (projectId: string, inputs: NewFileInput[]) =>
    invoke<MediaFile[]>("file_add_many", { projectId, inputs }),
  fileUpload: async (_projectId: string, _files: File[]): Promise<MediaFile[]> => {
    throw new Error("fileUpload is web-only — desktop uses fileAddMany");
  },
  fileRemove: (id: string) => invoke<void>("file_remove", { id }),
  fileReset: (id: string) => invoke<void>("file_reset", { id }),
```

**분석 / Analysis (Level 2)**
- 각 invoke 의 제네릭 타입(`invoke<Project[]>("project_list")`)이 Rust 반환 타입과 매칭되어야 함 — 런타임 검증 없이 신뢰. Rust 가 다른 모양을 보내면 TS 만 통과하고 런타임에 깨질 수 있다 (CWE-20 inadequate input validation).
- `fileUpload` 는 desktop 에선 의미 없으니 throw 로 명시. 인터페이스 일관성을 유지하기 위함.
- 인자명이 camelCase (projectId, fileIds) — Rust 측 serde 가 camelCase 로 받아들이게 설정되어 있어야 한다.

**짚어볼 점 / Watch-outs**
- 입력 검증 부재 — IPC 페이로드를 zod 등으로 런타임 파싱하면 더 견고. 일반적으로 desktop IPC 는 trusted boundary 라 생략 OK 이지만, 백엔드 변경 시 silent failure 가능.

```ts path=src/ipc/tauriClient.ts lines=32-45
  segmentsList: (fileId: string) =>
    invoke<Segment[]>("segments_list", { fileId }),

  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (settings: Settings) =>
    invoke<Settings>("settings_set", { settings }),

  pipelineRun: (fileIds: string[]) =>
    invoke<void>("pipeline_run", { fileIds }),

  translitPreview: (text: string) =>
    invoke<string>("translit_preview", { text }),

  mediaSrc: (file: MediaFile): string => convertFileSrc(file.path),
};
```

**분석 / Analysis (Level 2)**
- `settingsSet` 가 Settings 를 통째로 invoke — translateApiKey 포함. 데스크톱 IPC 는 같은 프로세스이므로 in-transit 노출은 작지만, 키가 Rust 측 SQLite 에 평문으로 저장될지 확인 필요.
- `mediaSrc` 는 동기 함수 — webview 가 asset:// URL 을 통해 직접 파일을 읽으므로 추가 IO 없음.

```ts path=src/ipc/tauriClient.ts lines=47-54
export function tauriOnPipelineEvent(
  handler: (e: PipelineEvent) => void,
): Promise<UnlistenFn> {
  return listen<PipelineEvent>("pipeline://event", (event) =>
    handler(event.payload),
  );
}
```

**분석 / Analysis (Level 2)**
- `pipeline://event` 채널 이름이 임의 문자열 — 백엔드(Rust) 측과 정확히 매칭되어야 한다. magic string 이므로 const 로 추출하면 안전.
- 반환 타입이 `Promise<UnlistenFn>` — Tauri 가 비동기로 listener 핸들을 돌려준다.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 도메인 타입과 정확히 매칭된 얇은 wrapper.
  - 인터페이스가 webIpc 와 짝을 이루어 surface-agnostic 코드를 가능하게 함.
- 위험 / Risks:
  - IPC 페이로드 런타임 검증 없음 (Rust 측과 silent drift 가능).
  - magic string `pipeline://event`.
  - API 키가 settingsSet 페이로드에 그대로 흘러감 — 저장 측 보안에 의존.
- 개선 권고 / Recommendations:
  - 채널 이름 상수화 (`const PIPELINE_CHANNEL = "pipeline://event"`).
  - 옵션: zod 등으로 IPC 응답 런타임 검증.
- 재읽기 권고 레벨 / Suggested re-read level: 1.
