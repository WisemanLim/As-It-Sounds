# src/store/useAppStore.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 앱 전역 상태 + IPC orchestration (프로젝트/파일/세그먼트). detectKind 유틸도 export.
- 진입점에서의 거리 / Distance from entrypoint: 2 (App → here, 그리고 거의 모든 컴포넌트가 사용)
- 라인 수 / LOC: 187
- 주요 export / Exports: `useAppStore`, `detectKind`
- 의존 모듈 / Depends on (internal): `../ipc/client`, `../lib/filePicker`, `../types`
- 사용된 외부 패키지 / External packages used: `zustand`
- 부수효과 / Side effects: IPC 명령 호출 (projectCreate/Delete/Rename, fileAddMany/Upload/Remove/Reset, ...)
- 동시성 모델 / Concurrency model: zustand set + async actions. 각 액션은 await 후 set 한다.
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/store/useAppStore.ts lines=1-10
import { create } from "zustand";
import { ipc } from "../ipc/client";
import type { PickedMedia } from "../lib/filePicker";
import type {
  FileStatus, MediaFile, MediaKind, Project, Segment,
} from "../types";
```

**분석 / Analysis (Level 2)**
- `import type` 로 타입만 가져옴 → 트리쉐이킹 친화.
- 도메인 모델(types.ts) + IPC client + PickedMedia 가 store 의 외부 의존성.

```ts path=src/store/useAppStore.ts lines=12-35
interface AppState {
  projects: Project[];
  files: MediaFile[];
  segments: Segment[];
  activeProjectId: string | null;
  activeFileId: string | null;
  loaded: boolean;

  bootstrap: () => Promise<void>;

  createProject: (name?: string) => Promise<Project>;
  renameProject: (id: string, name: string) => Promise<void>;
  deleteProject: (id: string) => Promise<void>;
  setActiveProject: (id: string | null) => Promise<void>;

  addFiles: (projectId: string, picked: PickedMedia) => Promise<MediaFile[]>;
  removeFile: (id: string) => Promise<void>;
  setActiveFile: (id: string | null) => Promise<void>;
  resetFile: (id: string) => Promise<void>;

  patchFile: (id: string, patch: Partial<MediaFile>) => void;
  upsertSegment: (segment: Segment) => void;
  clearSegmentsForFile: (fileId: string) => void;
}
```

**분석 / Analysis (Level 2)**
- 데이터(상태) + 액션(메서드) 한 인터페이스에 모음 — zustand 권장 패턴.
- 세 부류로 분류 가능: 부트스트랩, 프로젝트/파일 mutation(IPC 동반), patch/upsert/clear(이벤트 수신용 sync).
- `setActiveFile` 가 async 시그니처이지만 본문은 sync(아래 라인 145). over-declared 시그니처.

```ts path=src/store/useAppStore.ts lines=37-58
export const useAppStore = create<AppState>()((set) => ({
  projects: [],
  files: [],
  segments: [],
  activeProjectId: null,
  activeFileId: null,
  loaded: false,

  async bootstrap() {
    const projects = await ipc.projectList();
    set({ projects, loaded: true });
  },

  async createProject(name) {
    const project = await ipc.projectCreate(name);
    set((s) => ({
      projects: [project, ...s.projects],
      activeProjectId: project.id,
      activeFileId: null,
    }));
    return project;
  },
```

**분석 / Analysis (Level 2)**
- `bootstrap` 은 projectList 만 호출. 파일/세그먼트는 setActiveProject 진입 시점에 lazy 로드.
- `createProject` 가 새로 만든 프로젝트를 맨 앞에 prepend + active 로 설정.

**짚어볼 점 / Watch-outs**
- `bootstrap` 의 실패는 App.tsx 가 처리하지만, 그 외 액션 실패는 throw 가 그대로 호출자에게 전달 → 호출자가 catch 안 하면 unhandledrejection → fatal 오버레이.

```ts path=src/store/useAppStore.ts lines=60-83
  async renameProject(id, name) {
    await ipc.projectRename(id, name);
    set((s) => ({
      projects: s.projects.map((p) =>
        p.id === id ? { ...p, name, updatedAt: Date.now() } : p,
      ),
    }));
  },

  async deleteProject(id) {
    await ipc.projectDelete(id);
    set((s) => {
      const fileIds = s.files.filter((f) => f.projectId === id).map((f) => f.id);
      const nextProjects = s.projects.filter((p) => p.id !== id);
      return {
        projects: nextProjects,
        files: s.files.filter((f) => f.projectId !== id),
        segments: s.segments.filter((seg) => !fileIds.includes(seg.fileId)),
        activeProjectId:
          s.activeProjectId === id ? nextProjects[0]?.id ?? null : s.activeProjectId,
        activeFileId: fileIds.includes(s.activeFileId ?? "") ? null : s.activeFileId,
      };
    });
  },
```

**분석 / Analysis (Level 2)**
- `deleteProject` 가 cascading cleanup — 프로젝트 + 파일 + 세그먼트 + active 상태까지 정합성 유지. 견고.
- `fileIds.includes(s.activeFileId ?? "")` — null/undefined activeFileId 를 빈 문자열로 캐스팅하는 트릭. `""` 가 절대 fileIds 에 없다고 가정.

**짚어볼 점 / Watch-outs**
- `nextProjects[0]?.id` 가 가장 최근 프로젝트로 자동 fallback — UX 가 자동 점프하지만 사용자가 의도하지 않을 수 있다.

```ts path=src/store/useAppStore.ts lines=85-109
  async setActiveProject(id) {
    set({ activeProjectId: id, activeFileId: null });
    if (!id) {
      set({ files: [], segments: [] });
      return;
    }
    const files = await ipc.fileList(id);
    set((s) => ({
      files: [
        ...s.files.filter((f) => f.projectId !== id),
        ...files,
      ],
    }));

    const segmentsByFile = await Promise.all(
      files.map((f) => ipc.segmentsList(f.id).then((segs) => ({ id: f.id, segs }))),
    );
    set((s) => {
      const keep = s.segments.filter(
        (seg) => !files.some((f) => f.id === seg.fileId),
      );
      const fresh = segmentsByFile.flatMap((r) => r.segs);
      return { segments: [...keep, ...fresh] };
    });
  },
```

**분석 / Analysis (Level 2)**
- 활성 프로젝트로 전환 시 파일/세그먼트 fetch 의 두 단계 set:
  1. files: 다른 프로젝트 파일은 유지 + 새 프로젝트 파일 합침.
  2. segments: 새 프로젝트 파일의 세그먼트만 갱신.
- `Promise.all` 로 세그먼트 병렬 fetch.
- **레이스 컨디션**: 빠르게 두 프로젝트를 전환하면 첫 setActiveProject 의 await fileList 가 늦게 resolve 되어 두 번째 호출 후 stale data 를 set 할 수 있다. 마지막 set 이 첫 호출의 응답일 가능성.

**짚어볼 점 / Watch-outs**
- **CWE-362 race condition**: 다중 setActiveProject 동시 호출 시 stale-write 가능. activeProjectId 와 응답을 매번 비교(또는 AbortController)로 막아야 안전.

```ts path=src/store/useAppStore.ts lines=111-134
  async addFiles(projectId, picked) {
    let created: MediaFile[];
    if (picked.mode === "paths") {
      if (picked.inputs.length === 0) return [];
      const payload = picked.inputs.map((i) => ({
        name: i.name,
        path: i.path,
        kind: i.kind ?? detectKind(i.name),
        sizeBytes: i.sizeBytes,
      }));
      created = await ipc.fileAddMany(projectId, payload);
    } else {
      if (picked.files.length === 0) return [];
      created = await ipc.fileUpload(projectId, picked.files);
    }
    set((s) => ({
      files: [...s.files, ...created],
      activeFileId: s.activeFileId ?? created[0]?.id ?? null,
      projects: s.projects.map((p) =>
        p.id === projectId ? { ...p, updatedAt: Date.now() } : p,
      ),
    }));
    return created;
  },
```

**분석 / Analysis (Level 2)**
- discriminated union 으로 paths/files 분기 처리.
- `activeFileId: s.activeFileId ?? created[0]?.id ?? null` — 이미 active 가 있으면 유지, 없으면 첫 새 파일 활성화. 합리적.
- 빈 픽 조기 반환 — HomePage 의 startWithPicked 와 중복 가드. 다중 방어.

```ts path=src/store/useAppStore.ts lines=136-157
  async removeFile(id) {
    await ipc.fileRemove(id);
    set((s) => ({
      files: s.files.filter((f) => f.id !== id),
      segments: s.segments.filter((seg) => seg.fileId !== id),
      activeFileId: s.activeFileId === id ? null : s.activeFileId,
    }));
  },

  async setActiveFile(id) {
    set({ activeFileId: id });
  },

  async resetFile(id) {
    await ipc.fileReset(id);
    set((s) => ({
      files: s.files.map((f) =>
        f.id === id ? { ...f, status: "pending" as FileStatus, error: undefined } : f,
      ),
      segments: s.segments.filter((seg) => seg.fileId !== id),
    }));
  },
```

**분석 / Analysis (Level 2)**
- `setActiveFile` 가 async 이지만 await 가 없다. 호출자에게 Promise 반환만 약속하는데 실제 내용은 sync. 시그니처 정리 여지.
- `resetFile` 도 cascading — 파일 상태와 세그먼트 함께 정리.

```ts path=src/store/useAppStore.ts lines=159-179
  patchFile(id, patch) {
    set((s) => ({
      files: s.files.map((f) => (f.id === id ? { ...f, ...patch } : f)),
    }));
  },

  upsertSegment(segment) {
    set((s) => {
      const others = s.segments.filter(
        (seg) => !(seg.fileId === segment.fileId && seg.index === segment.index),
      );
      return { segments: [...others, segment] };
    });
  },

  clearSegmentsForFile(fileId) {
    set((s) => ({
      segments: s.segments.filter((seg) => seg.fileId !== fileId),
    }));
  },
}));
```

**분석 / Analysis (Level 2)**
- `upsertSegment` 가 `(fileId, index)` 키로 중복 제거 → 같은 세그먼트 재발행 안전.
- `O(N)` 필터를 매 세그먼트마다 호출 → 큰 N 에서는 비용. zustand 상태에 Map 을 두면 O(1) 가능하지만 immer/zustand 패턴에서 보일러플레이트 증가.

**짚어볼 점 / Watch-outs**
- 1000+ 세그먼트 파일에서 매 세그먼트 이벤트마다 전체 배열 복사 → 성능 부담. 현실적 음성 클립은 수십~수백 세그먼트이므로 OK.

```ts path=src/store/useAppStore.ts lines=181-186
export function detectKind(name: string): MediaKind {
  const lower = name.toLowerCase();
  const videoExts = [".mp4", ".mov", ".mkv", ".webm", ".avi", ".m4v"];
  if (videoExts.some((ext) => lower.endsWith(ext))) return "video";
  return "audio";
}
```

**분석 / Analysis (Level 2)**
- 순수 유틸인데 store 모듈에 위치 — `lib/media.ts` 같은 자리로 옮기는 게 자연스러움.
- videoExts 가 EXTS 의 부분집합과 또 다르게 정의됨 → drift 위험 (확장자 정의의 4번째 사본).

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - cascade cleanup(deleteProject, removeFile, resetFile) 으로 데이터 정합성 유지.
  - discriminated union (`picked.mode`) 으로 desktop/web 추가 분기.
  - 작은 액션들이 명확한 책임 분리.
- 위험 / Risks:
  - **race condition**: 빠른 프로젝트 전환 시 stale 응답으로 잘못된 상태 가능.
  - `detectKind` / videoExts 가 4번째 확장자 정의 사본 (drift).
  - 세그먼트 upsert 가 O(N) — 대용량 데이터에선 부담.
  - `setActiveFile` 시그니처가 async 인데 본문 sync (시그니처 부풀림).
- 개선 권고 / Recommendations:
  - setActiveProject 에서 응답을 set 전에 `if (get().activeProjectId !== id) return` 가드.
  - `detectKind` 와 확장자 정의를 `lib/media.ts` 로 통합.
  - 세그먼트를 Map<string, Segment[]> 로 보관해 O(1) upsert.
- 재읽기 권고 레벨 / Suggested re-read level: 3 — race-condition 보강과 자료구조 변경은 영향 범위가 넓어 깊게 검토.
