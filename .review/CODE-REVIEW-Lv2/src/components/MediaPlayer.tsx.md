# src/components/MediaPlayer.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 오디오/비디오 재생기. forwardRef 로 부모(ProjectPage)에 seek/play/pause API 노출.
- 진입점에서의 거리 / Distance from entrypoint: 3
- 라인 수 / LOC: 92
- 주요 export / Exports: `MediaPlayer`, `MediaPlayerHandle` (type)
- 의존 모듈 / Depends on (internal): `../lib/cn`, `../lib/format`, `../types`
- 사용된 외부 패키지 / External packages used: `react`
- 부수효과 / Side effects: DOM media 엘리먼트 제어 (currentTime, play, pause)
- 동시성 모델 / Concurrency model: HTMLMediaElement 의 자체 이벤트 (timeupdate, loadedmetadata 등)
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/components/MediaPlayer.tsx lines=1-17
import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import { cn } from "../lib/cn";
import { formatTime } from "../lib/format";
import type { MediaFile } from "../types";

export interface MediaPlayerHandle {
  seek: (ms: number) => void;
  play: () => void;
  pause: () => void;
}

interface Props {
  file: MediaFile;
  src: string;
  onTimeUpdate?: (ms: number) => void;
  onDurationChange?: (ms: number) => void;
}
```

**분석 / Analysis (Level 2)**
- `forwardRef` 패턴 + `useImperativeHandle` 로 부모가 자식의 명령형 API를 호출하게 함. 일반적으로 React 에서 선호하지는 않지만 미디어 시크처럼 자연스러운 사용 사례.
- `MediaFile.kind` 로 audio vs video 분기 → 하나의 컴포넌트로 두 미디어 타입 모두 처리.

```tsx path=src/components/MediaPlayer.tsx lines=19-27
export const MediaPlayer = forwardRef<MediaPlayerHandle, Props>(function MediaPlayer(
  { file, src, onTimeUpdate, onDurationChange },
  ref,
) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const [_playing, setPlaying] = useState(false);

  const getEl = () => (file.kind === "video" ? videoRef.current : audioRef.current);
```

**분석 / Analysis (Level 2)**
- `forwardRef<MediaPlayerHandle, Props>` 의 generic 순서: <Handle, Props>.
- audio/video 양쪽 ref 를 동시에 보유. 사용되는 쪽은 file.kind 에 따라 하나뿐이지만, 미사용 ref 의 null 상태는 OK.
- `_playing` — 변수에 `_` prefix 로 "이 값은 직접 안 읽지만 setter 만 쓴다" 의도 표현. TS는 사용 안 함 경고를 안 띄움.

**짚어볼 점 / Watch-outs**
- `_playing` 을 정말 안 읽는다면 `useState` 자체가 불필요 → 리렌더 비용만 추가. `useRef` 로 충분.

```tsx path=src/components/MediaPlayer.tsx lines=29-36
  useImperativeHandle(ref, () => ({
    seek: (ms) => {
      const el = getEl();
      if (el) el.currentTime = ms / 1000;
    },
    play: () => getEl()?.play(),
    pause: () => getEl()?.pause(),
  }));
```

**분석 / Analysis (Level 2)**
- `play()` 는 Promise 를 반환하지만 호출 측에서 await 하지 않음 → autoplay policy 위반 시 unhandled rejection 가능 (main.tsx fatal 오버레이 트리거 위험).
- 두 번째 인자(deps)가 없어 매 렌더마다 새 핸들 객체 생성 — 사용 측이 ref.current 만 쓰면 큰 문제는 아니지만 권장 패턴은 deps 명시.

**짚어볼 점 / Watch-outs**
- 브라우저 autoplay 정책: 사용자 제스처 없이 play() 호출 시 NotAllowedError 거부 → fatal 오버레이로 빠짐. `play()?.catch(() => {})` 권장.

```tsx path=src/components/MediaPlayer.tsx lines=38-42
  useEffect(() => {
    setPlaying(false);
    const el = getEl();
    if (el) el.currentTime = 0;
  }, [src, file.kind]);
```

**분석 / Analysis (Level 2)**
- src 또는 kind 가 바뀌면 재생 위치를 0 으로, 재생 상태도 false 로. 파일 전환 UX 자연스러움.
- 의존성에 `getEl` 이 없지만 closure로 최신 ref 를 잡는다 → 안전.

```tsx path=src/components/MediaPlayer.tsx lines=44-51
  const handleTimeUpdate = (e: React.SyntheticEvent<HTMLMediaElement>) => {
    onTimeUpdate?.(Math.floor(e.currentTarget.currentTime * 1000));
  };

  const handleDuration = (e: React.SyntheticEvent<HTMLMediaElement>) => {
    const d = e.currentTarget.duration;
    if (isFinite(d)) onDurationChange?.(Math.floor(d * 1000));
  };
```

**분석 / Analysis (Level 2)**
- `isFinite(d)` 가드 — 일부 스트리밍 미디어는 duration 이 `Infinity` 일 수 있음. 좋은 방어.
- `Math.floor(... * 1000)` 으로 ms 정수화 — 도메인 타입과 일치.
- `timeupdate` 이벤트는 250~500ms 마다 발화 — 큰 리스트 + 매 이벤트마다 부모 setState → 잠재적 리렌더 폭증. 현재는 SegmentView 의 active 계산이 가벼워 OK.

```tsx path=src/components/MediaPlayer.tsx lines=53-91
  return (
    <div className={cn(
      "overflow-hidden rounded-xl border border-line bg-black",
      file.kind === "audio" && "bg-bg-elevated p-4",
    )}>
      {file.kind === "video" ? (
        <video ref={videoRef} src={src} controls
          onTimeUpdate={handleTimeUpdate}
          onLoadedMetadata={handleDuration}
          onPlay={() => setPlaying(true)}
          onPause={() => setPlaying(false)}
          className="aspect-video w-full bg-black" />
      ) : (
        <div className="flex flex-col gap-2">
          <div className="truncate text-sm font-medium text-fg">{file.name}</div>
          <audio ref={audioRef} src={src} controls
            onTimeUpdate={handleTimeUpdate}
            onLoadedMetadata={handleDuration}
            onPlay={() => setPlaying(true)}
            onPause={() => setPlaying(false)}
            className="w-full" />
          <div className="text-right font-mono text-[11px] text-fg-subtle">
            {formatTime((audioRef.current?.currentTime ?? 0) * 1000)}
          </div>
        </div>
      )}
    </div>
  );
});
```

**분석 / Analysis (Level 2)**
- 네이티브 `controls` 속성에 의존 → 브라우저 기본 UI 사용으로 코드 단순.
- 오디오 영역의 시간 표시가 `audioRef.current?.currentTime` 을 직접 읽음 → React 의 렌더 사이클과 동기화되지 않아 항상 한 박자 늦은 값(또는 0)이 표시될 수 있다.

**짚어볼 점 / Watch-outs**
- **시간 표시 동기화 문제**: ref 의 currentTime 은 React state 가 아니므로 timeupdate 이벤트로 강제 리렌더가 일어나지 않는 한 갱신이 안 보인다. 현재 `handleTimeUpdate` 가 부모에게 통지하고, 부모가 자식 props 를 변경시키지 않으므로 자체 리렌더가 안 일어남 → 표시 값이 0:00 에 멈출 가능성. 부모의 currentTimeMs prop 을 받아 표시하는 것이 정석.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - audio/video 단일 컴포넌트 통합.
  - `isFinite(d)` 같은 방어 가드.
  - `seek` 명령형 API를 useImperativeHandle 로 깔끔히 노출.
- 위험 / Risks:
  - `play()` 의 Promise rejection 미처리 → autoplay 정책 위반 시 fatal 오버레이로 빠짐.
  - audio 모드의 시간 표시가 ref 직접 읽기로 신뢰성 낮음.
  - 미사용 `_playing` state → 불필요한 리렌더 가능.
- 개선 권고 / Recommendations:
  - `play: () => getEl()?.play().catch(() => {})` 로 rejection 흡수.
  - 시간 표시를 부모로부터 `currentTimeMs` prop 으로 받기 (이미 ProjectPage 가 가지고 있음).
  - `_playing` 를 정말 사용 안 한다면 제거.
- 재읽기 권고 레벨 / Suggested re-read level: 2 — 동기화/Promise 처리 보완 후 재확인.
