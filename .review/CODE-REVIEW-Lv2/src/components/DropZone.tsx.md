# src/components/DropZone.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: HTML5 드래그&드롭 + 클릭 트리거를 받는 파일 업로드 영역.
- 진입점에서의 거리 / Distance from entrypoint: 2 (HomePage / ProjectPage 가 사용)
- 라인 수 / LOC: 93
- 주요 export / Exports: `DropZone` (named)
- 의존 모듈 / Depends on (internal): `./icons`, `../lib/cn`, `../lib/filePicker`
- 사용된 외부 패키지 / External packages used: `react`
- 부수효과 / Side effects: 없음 (콜백 props 호출만)
- 동시성 모델 / Concurrency model: React 상태 + 동기 이벤트 핸들러
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/components/DropZone.tsx lines=1-11
import { useCallback, useState } from "react";
import { IconUpload, IconAudio, IconVideo } from "./icons";
import { cn } from "../lib/cn";
import { isAllowedMediaName, type PickedMedia } from "../lib/filePicker";

interface DropZoneProps {
  onFiles: (picked: PickedMedia) => void;
  onPick?: () => void;
  compact?: boolean;
}
```

**분석 / Analysis (Level 2)**
- `onFiles` 는 드래그 드롭으로 받은 파일 배열을 부모에게 알린다. `onPick` 은 클릭 시 호출 (네이티브 파일 선택기 트리거 용도).
- `PickedMedia` 는 desktop의 paths/web의 File[] 두 모드를 표현하는 union 타입.
- `compact` 는 사이드바에 작게 표시할 때를 위한 토글.

```tsx path=src/components/DropZone.tsx lines=12-25
export function DropZone({ onFiles, onPick, compact }: DropZoneProps) {
  const [isOver, setIsOver] = useState(false);

  const handleFiles = useCallback(
    (fileList: FileList | File[]) => {
      const picked: File[] = [];
      for (const f of Array.from(fileList)) {
        if (!isAllowedMediaName(f.name)) continue;
        picked.push(f);
      }
      if (picked.length > 0) onFiles({ mode: "files", files: picked });
    },
    [onFiles],
  );
```

**분석 / Analysis (Level 2)**
- 드롭된 파일에서 `isAllowedMediaName`으로 확장자 필터링. 화이트리스트 패턴(EXTS 세트).
- `Array.from(fileList)`: FileList 가 iterable이지만 표준 배열 메서드(filter 등)가 부족하므로 배열로 변환.
- `mode: "files"` 로 항상 보낸다 — 이 컴포넌트는 항상 브라우저-스타일 File 만 다루기 때문 (Tauri 네이티브 드롭은 useTauriDrop이 별도로 처리).

**짚어볼 점 / Watch-outs**
- 모두 필터링되어 0개가 되면 콜백 미호출 → 사용자에게 피드백이 없다. "지원하지 않는 파일" 토스트가 있으면 더 친절.

```tsx path=src/components/DropZone.tsx lines=27-41
  const onDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsOver(true);
  };
  const onDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsOver(false);
  };
  const onDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsOver(false);
    if (e.dataTransfer.files?.length) {
      handleFiles(e.dataTransfer.files);
    }
  };
```

**분석 / Analysis (Level 2)**
- `e.preventDefault()` — 브라우저가 파일을 새 탭에서 열어버리는 기본 동작을 막는다. 빠뜨리기 쉬운 함정.
- `dataTransfer.files` 가 있을 때만 처리 → URL 드롭 등 다른 dataTransfer 종류는 무시.

**짚어볼 점 / Watch-outs**
- `dragleave`는 자식 요소를 지날 때마다 발생해 상태가 깜빡일 수 있다(흔한 디자인 함정). 현재는 단순 토글이라 가끔 깜빡임 가능.

```tsx path=src/components/DropZone.tsx lines=43-92
  return (
    <div
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      onClick={onPick}
      className={cn(
        "group flex cursor-pointer flex-col items-center justify-center rounded-2xl border border-dashed text-center transition-colors",
        isOver
          ? "border-brand bg-brand-soft/30"
          : "border-line bg-bg-elevated hover:border-fg-subtle hover:bg-bg-hover",
        compact ? "gap-2 px-4 py-6" : "gap-4 px-8 py-16",
      )}
    >
      <div
        className={cn(
          "flex items-center justify-center rounded-full bg-brand-soft text-brand transition-transform group-hover:scale-105",
          compact ? "h-10 w-10" : "h-16 w-16",
        )}
      >
        <IconUpload width={compact ? 18 : 28} height={compact ? 18 : 28} />
      </div>

      <div className={cn(compact ? "space-y-0.5" : "space-y-1.5")}>
        <p className={cn("font-medium text-fg", compact ? "text-sm" : "text-lg")}>
          {isOver ? "여기에 놓아 업로드하세요" : "음성 · 영상 파일을 끌어다 놓으세요"}
        </p>
        <p className={cn("text-fg-muted", compact ? "text-xs" : "text-sm")}>
          또는 클릭해서 선택 · MP3, WAV, M4A, MP4, MOV, MKV 등 지원
        </p>
      </div>

      {!compact && (
        <div className="mt-4 flex items-center gap-3 text-xs text-fg-subtle">
          <span className="inline-flex items-center gap-1">
            <IconAudio width={14} height={14} />
            오디오
          </span>
          <span className="h-3 w-px bg-line" />
          <span className="inline-flex items-center gap-1">
            <IconVideo width={14} height={14} />
            비디오
          </span>
          <span className="h-3 w-px bg-line" />
          <span>여러 파일 동시 업로드</span>
        </div>
      )}
    </div>
  );
}
```

**분석 / Analysis (Level 2)**
- `cn(...)` 으로 조건부 클래스 합성 — Tailwind 가독성 향상의 표준 패턴.
- `<div onClick={onPick}>` 가 의미상 버튼이지만 마크업은 div → 접근성 측면에서는 `role="button"` + `tabIndex={0}` + 키보드 이벤트가 권장된다.
- 데스크톱 안내 텍스트 "MP3, WAV, M4A, MP4, MOV, MKV 등 지원" 이 filePicker EXTS 와 별도로 하드코딩 → 확장자 추가 시 두 곳을 함께 바꿔야 함 (drift 위험).

**짚어볼 점 / Watch-outs**
- 접근성: 키보드만 쓰는 사용자가 `Enter`로 트리거 불가.
- DOM의 `<input type="file" hidden>` 없이 onPick 만으로 동작하므로 React/Tauri의 picker에 의존. 키보드/스크린리더 경로가 빈약.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - 화이트리스트 확장자 검증 → 잘못된 파일 차단 (CWE-434, unrestricted file upload 완화).
  - compact 모드 분기로 재사용성 좋음.
- 위험 / Risks:
  - 0개 필터링 시 사용자 피드백 없음.
  - 접근성(키보드/스크린리더) 미흡.
  - 지원 확장자 목록이 두 곳에 하드코딩 (drift).
- 개선 권고 / Recommendations:
  - `picked.length === 0 && fileList.length > 0` 일 때 "지원하지 않는 파일" 토스트.
  - `role="button" tabIndex={0} onKeyDown` 추가.
  - 지원 확장자 문자열을 `filePicker.ts` 의 EXTS 에서 파생.
- 재읽기 권고 레벨 / Suggested re-read level: 1 — UI 복잡도가 낮고 흐름이 명확.
