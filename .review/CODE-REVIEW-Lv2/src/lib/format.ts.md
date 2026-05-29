# src/lib/format.ts — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: ms 시간/상대시간 포맷팅 유틸.
- 진입점에서의 거리 / Distance from entrypoint: 2 (FileList, SegmentView, MediaPlayer 사용)
- 라인 수 / LOC: 37
- 주요 export / Exports: `formatTime`, `formatDuration`, `formatRelative`
- 의존 모듈 / Depends on (internal): 없음
- 사용된 외부 패키지 / External packages used: 없음
- 부수효과 / Side effects: `Date.now()` 호출 (formatRelative)
- 동시성 모델 / Concurrency model: N/A
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```ts path=src/lib/format.ts lines=1-14
export function formatTime(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const millis = Math.floor(ms % 1000);

  const pad = (n: number, len = 2) => n.toString().padStart(len, "0");

  if (hours > 0) {
    return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}.${pad(millis, 3)}`;
  }
  return `${pad(minutes)}:${pad(seconds)}.${pad(millis, 3)}`;
}
```

**분석 / Analysis (Level 2)**
- `Math.floor(ms % 1000)` — 음수 ms 가 들어오면 millis 가 음수가 될 수 있다. `Math.abs` 가드는 없음. 백엔드가 양수만 보내준다고 가정.
- 시간 단위는 SegmentView 의 미세 시각 표시용 — millisecond 까지 정밀 표시.
- `pad` 가 클로저로 매 호출마다 새로 생성 — 미세한 비용. 모듈 스코프로 빼면 약간 더 효율적.

**짚어볼 점 / Watch-outs**
- 음수 ms 보호 없음 (방어적 코드는 부족하지만 도메인이 그럴 일이 없음).

```ts path=src/lib/format.ts lines=16-22
export function formatDuration(ms?: number): string {
  if (ms == null) return "--:--";
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
```

**분석 / Analysis (Level 2)**
- 옵셔널 ms 를 받아 null 일 때 `"--:--"` placeholder.
- mm:ss 만 표시 (시간 단위 무시) — 보통 음성 클립이 1시간을 넘지 않는다는 가정.
- `ms == null` (loose equality) 으로 undefined/null 모두 처리.

**짚어볼 점 / Watch-outs**
- 1시간 초과 음성: 60:00, 70:00 등으로 표시 — 잘못은 아니지만 사용자가 시간 단위를 기대하면 헷갈림.

```ts path=src/lib/format.ts lines=24-36
export function formatRelative(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "방금 전";
  if (mins < 60) return `${mins}분 전`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}시간 전`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}일 전`;
  const d = new Date(timestamp);
  return `${d.getFullYear()}-${(d.getMonth() + 1).toString().padStart(2, "0")}-${d.getDate().toString().padStart(2, "0")}`;
}
```

**분석 / Analysis (Level 2)**
- 미래 timestamp (diff < 0) 면 `mins < 1` → "방금 전" 으로 폴백. 합리적.
- 7일 이상은 절대 날짜 표시. 로케일 영향을 안 받는 ISO 풍 형식.
- 그러나 export 됐지만 실제 사용처가 보이지 않음 → dead code 가능.

**짚어볼 점 / Watch-outs**
- `formatRelative` 가 사용 안 된다면 번들 부담은 작지만 유지비용 발생. 실제 호출 여부 grep 으로 확인 필요.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 의존성 0, 가독성 높음, ms placeholder 명시.
  - Korean 친화적 표현 ("방금 전" 등).
- 위험 / Risks:
  - `formatRelative` 의 사용처가 불분명 → dead code 가능성.
  - 1시간 초과 음성에 `formatDuration` 결과가 60+ 분으로 표시.
- 개선 권고 / Recommendations:
  - 사용처 grep 후 미사용 함수 제거 또는 사용처 확보.
  - `formatDuration` 에 시간 단위 분기 추가.
- 재읽기 권고 레벨 / Suggested re-read level: 0.
