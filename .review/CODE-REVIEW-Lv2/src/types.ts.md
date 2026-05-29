# src/types.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 프론트엔드 도메인 모델 + IPC 페이로드 타입의 단일 출처(single source of truth).
- 진입점에서의 거리 / Distance from entrypoint: 1 (App/Components 가 직접 import)
- 라인 수 / LOC: 63
- 주요 export / Exports: `MediaKind`, `FileStatus`, `MediaFile`, `Segment`, `Project`, `Settings`, `NewFileInput`, `PipelineEvent`
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: 없음
- 부수효과 / Side effects: 없음 (타입 전용)
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 타입 정의이므로 컴파일 검사가 곧 검증.

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/types.ts lines=1-2
export type MediaKind = "audio" | "video";
export type FileStatus = "pending" | "processing" | "done" | "error";
```

**분석 / Analysis (Level 2)**
- 문자열 리터럴 유니온 타입 — enum 대신 사용해 런타임 코스트 0, JSON 직렬화 그대로.
- FileStatus 4가지는 파이프라인의 자연스러운 라이프사이클: 대기 → 처리 중 → 완료/오류.

**짚어볼 점 / Watch-outs**
- "cancelled" 상태가 없음 → 사용자 취소를 도입하려면 enum 확장 필요.

```ts path=src/types.ts lines=4-15
export interface MediaFile {
  id: string;
  projectId: string;
  name: string;
  path: string;
  kind: MediaKind;
  sizeBytes?: number;
  durationMs?: number;
  status: FileStatus;
  error?: string;
  createdAt: number;
}
```

**분석 / Analysis (Level 2)**
- `path`는 desktop의 절대 경로 또는 web의 server-side 경로. 두 모드 공용 — 타입은 단순화되어 있지만 의미는 surface별로 다르다는 점이 숨은 함정.
- `createdAt`은 Unix epoch ms (`Date.now()` 와 호환).
- 옵셔널 필드들이 명시적이라 백엔드가 일부만 보내도 컴파일이 깨지지 않는다.

**짚어볼 점 / Watch-outs**
- `path`의 의미가 surface별로 다르지만 같은 필드명을 공유 → 잘못된 곳에서 `fs.stat(path)` 하면 web에서 에러.
- `sizeBytes`/`durationMs` 가 옵셔널이라 UI가 항상 `null/undefined` 케이스를 처리해야 함 (실제로 FileList/format.ts 가 처리함).

```ts path=src/types.ts lines=17-26
export interface Segment {
  id: string;
  fileId: string;
  index: number;
  startMs: number;
  endMs: number;
  textEn: string;
  textHangul?: string;
  textKo?: string;
}
```

**분석 / Analysis (Level 2)**
- 세그먼트는 항상 영어 원문(`textEn`)을 가지고, 발음 한글 표기(`textHangul`)와 한국어 번역(`textKo`)은 옵셔널.
- `index` 필드로 정렬/upsert 가능 → store의 `upsertSegment`가 `(fileId, index)` 키로 중복을 제거한다.
- `start/end Ms` 는 정수형으로 가정 — `formatTime`이 floor 처리.

```ts path=src/types.ts lines=28-33
export interface Project {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
}
```

**분석 / Analysis (Level 2)**
- 최소한의 프로젝트 메타데이터. `updatedAt`은 store가 클라이언트에서도 갱신.

```ts path=src/types.ts lines=35-41
export interface Settings {
  whisperModel: string;
  translateEndpoint: string;
  translateApiKey?: string | null;
  translator: string;
  ffmpegPath?: string | null;
}
```

**분석 / Analysis (Level 2)**
- API 키가 도메인 타입에 포함됨 → 메모리 내 평문 보유 (CWE-312 / cleartext storage). 데스크톱 앱이라 OS 키체인 활용이 이상적이지만 현 구조에선 SQLite 평문 가능성 큼 (Rust 측 확인 필요).
- `translator` 가 string 타입이라 `"mymemory" | "libretranslate" | "none"` 같은 제약이 없다 → 오타 시 컴파일이 잡지 못한다.

**짚어볼 점 / Watch-outs**
- 보안: API 키는 `Settings` 객체에 직접 들어있어 toast/log 같은 부주의한 출력 시 노출 가능. UI 코드에서 `JSON.stringify(settings)` 같은 호출이 없는지 검토 필요.

```ts path=src/types.ts lines=43-48
export interface NewFileInput {
  name: string;
  path: string;
  kind?: MediaKind;
  sizeBytes?: number;
}
```

**분석 / Analysis (Level 2)**
- 데스크톱 IPC 페이로드. 백엔드가 `kind`를 옵셔널로 받고 누락 시 추론.
- `MediaFile`과 닮았지만 작은 부분 집합 — 신규 추가용 DTO.

```ts path=src/types.ts lines=50-62
export type PipelineEvent =
  | { type: "fileStarted"; fileId: string }
  | { type: "modelDownloading"; progress: number; message: string }
  | {
      type: "progress";
      fileId: string;
      stage: "decode" | "asr" | "translate" | string;
      message: string;
    }
  | { type: "segment"; segment: Segment }
  | { type: "fileDone"; fileId: string; durationMs?: number }
  | { type: "fileError"; fileId: string; error: string }
  | { type: "allDone" };
```

**분석 / Analysis (Level 2)**
- discriminated union — `e.type` 분기에서 TS가 자동으로 narrowing.
- `stage`가 `"decode" | "asr" | "translate" | string` 형태로 union에 `string`이 들어가 있어 사실상 자유 문자열. 자동완성 힌트만 주는 트릭이지만 타입 안전성은 잃었다.
- `modelDownloading`은 파일과 무관한 글로벌 이벤트로 따로 분기.

**짚어볼 점 / Watch-outs**
- `progress` 의 `stage`가 `string`까지 허용 → 백엔드와 단계 이름이 어긋나도 타입 검사가 통과. 엄격한 리터럴만 두거나 `as const` 매핑이 안전.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - discriminated union으로 IPC 이벤트가 깔끔히 narrowing 된다.
  - 도메인 모델이 한 파일에 모여 있어 변경 추적이 쉽다.
- 위험 / Risks:
  - `Settings.translateApiKey` 가 평문 string으로 떠다닌다 (보안 검토 대상).
  - `translator`, `progress.stage` 가 자유 문자열 — 오타가 컴파일을 통과.
- 개선 권고 / Recommendations:
  - `translator`를 `'mymemory' | 'libretranslate' | 'none'` 리터럴 유니온으로 좁힐 것.
  - `progress.stage`에서 `| string` 제거 또는 별도 `KnownStage` 타입을 정의.
  - API 키를 별도 secrets 객체로 분리해 로깅 누락 위험을 줄일 것.
- 재읽기 권고 레벨 / Suggested re-read level: 0 — 짧고 명확. 도메인이 바뀔 때만 다시 보면 된다.
