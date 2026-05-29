# src/pages/SettingsPage.tsx — 코드 리뷰 / Code Review

> 리뷰 레벨 / Review Level: 2
> [← INDEX.md](../../INDEX.md)

**파일 요약 / File Summary**
- 역할 / Role: 설정 페이지 — Whisper 모델/번역 엔진/ffmpeg 경로 변경.
- 진입점에서의 거리 / Distance from entrypoint: 3 (App → Routes → here)
- 라인 수 / LOC: 156
- 주요 export / Exports: `SettingsPage`
- 의존 모듈 / Depends on (internal): `../ipc/client`, `../types`, `../lib/toast`
- 사용된 외부 패키지 / External packages used: `react`
- 부수효과 / Side effects: IPC 호출(settingsGet/Set), toast
- 동시성 모델 / Concurrency model: async save — 매 onChange 시 호출
- 테스트 커버리지 흔적 / Test coverage signal: 없음

## Step 5 — 라인별 분석 / Line-by-line walk

```tsx path=src/pages/SettingsPage.tsx lines=1-27
import { useEffect, useState } from "react";
import { ipc } from "../ipc/client";
import type { Settings } from "../types";
import toast from "../lib/toast";

const MODEL_OPTIONS: Array<{ value: string; label: string }> = [
  { value: "tiny.en", label: "tiny.en (75 MB · 가장 빠름)" },
  { value: "base.en", label: "base.en (142 MB · 기본 권장)" },
  { value: "small.en", label: "small.en (466 MB · 균형)" },
  { value: "medium.en", label: "medium.en (1.5 GB · 정확)" },
  { value: "large-v3", label: "large-v3 (2.9 GB · 최고 품질)" },
];

const TRANSLATOR_OPTIONS: Array<{ value: string; label: string; endpoint: string }> = [
  {
    value: "mymemory",
    label: "MyMemory (무료, 키 불필요, 1000단어/일)",
    endpoint: "https://api.mymemory.translated.net/get",
  },
  ...
];
```

**분석 / Analysis (Level 2)**
- 옵션 데이터를 컴포넌트 외부 const 로 둠 → 매 렌더마다 재생성 안 됨.
- 한국어 label 이 직접 박혀 있음 — i18n 미고려.
- 외부 번역 서비스 URL 이 코드에 노출 — 비밀이 아니므로 OK.

```tsx path=src/pages/SettingsPage.tsx lines=28-42
export function SettingsPage() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    ipc.settingsGet().then(setSettings);
  }, []);

  if (!settings) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-fg-muted">
        설정 불러오는 중...
      </div>
    );
  }
```

**분석 / Analysis (Level 2)**
- 마운트 시 한 번만 fetch. catch 없음 → settingsGet 거부 시 unhandledrejection 으로 → main.tsx fatal 오버레이.
- 로딩 상태(`settings === null`)와 정상 상태를 명시적으로 분기.

**짚어볼 점 / Watch-outs**
- `ipc.settingsGet().then(setSettings)` 에 `.catch` 가 없음 → 실패 시 사용자에게 무한 "불러오는 중..." + 콘솔 fatal.

```tsx path=src/pages/SettingsPage.tsx lines=44-54
  const save = async (next: Settings) => {
    setSaving(true);
    try {
      const saved = await ipc.settingsSet(next);
      setSettings(saved);
    } catch (e) {
      toast(`설정 저장 실패: ${String(e)}`);
    } finally {
      setSaving(false);
    }
  };
```

**분석 / Analysis (Level 2)**
- save 가 매 onChange 마다 호출 → 사용자가 endpoint 를 한 글자씩 타이핑하면 매 키스트로크마다 IPC 호출 (debounce 없음).
- `String(e)` — Error 객체일 때 `"Error: ..."` 형식. 사용자에게 약간 불친절하지만 OK.
- `saving` 플래그가 있지만 UI 비활성화는 안 함 → 동시 저장 가능.

**짚어볼 점 / Watch-outs**
- **디바운스 부재**: endpoint/ffmpeg path 입력 시 키 입력마다 백엔드 호출. 네트워크 부하/race condition 위험.

```tsx path=src/pages/SettingsPage.tsx lines=56-95
  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-line px-6">
        <h1 className="text-sm font-semibold">설정</h1>
        {saving && <span className="text-xs text-fg-muted">저장 중...</span>}
      </header>
      <div className="mx-auto w-full max-w-2xl px-6 py-8 space-y-6 text-sm">
        <Field label="Whisper 모델" help="첫 사용 시 자동 다운로드됩니다.">
          <select
            ...
            value={settings.whisperModel}
            onChange={(e) => save({ ...settings, whisperModel: e.target.value })}
          >
            {MODEL_OPTIONS.map((m) => (
              <option key={m.value} value={m.value}>{m.label}</option>
            ))}
          </select>
        </Field>

        <Field label="번역 엔진">
          <select
            ...
            value={settings.translator}
            onChange={(e) => {
              const opt = TRANSLATOR_OPTIONS.find((o) => o.value === e.target.value);
              save({
                ...settings,
                translator: e.target.value,
                translateEndpoint: opt?.endpoint ?? settings.translateEndpoint,
              });
            }}
          >
            ...
          </select>
```

**분석 / Analysis (Level 2)**
- select 변경 시 default endpoint 함께 갱신 — UX 친절.
- 컨트롤된 입력(controlled input) — value/onChange 페어로 SSR/HMR 안전.

```tsx path=src/pages/SettingsPage.tsx lines=96-125
          <input
            className="mt-2 w-full rounded-lg border border-line bg-bg px-3 py-2 font-mono text-xs"
            value={settings.translateEndpoint}
            onChange={(e) =>
              save({ ...settings, translateEndpoint: e.target.value })
            }
            placeholder="번역 엔드포인트 URL"
          />
          {settings.translator === "libretranslate" && (
            <input
              className="mt-2 w-full rounded-lg border border-line bg-bg px-3 py-2 font-mono text-xs"
              value={settings.translateApiKey ?? ""}
              onChange={(e) =>
                save({ ...settings, translateApiKey: e.target.value || null })
              }
              placeholder="LibreTranslate API Key (선택)"
            />
          )}
        </Field>

        <Field label="ffmpeg 경로" help="비워두면 PATH 에서 자동으로 찾습니다.">
          <input
            ...
            value={settings.ffmpegPath ?? ""}
            onChange={(e) =>
              save({ ...settings, ffmpegPath: e.target.value || null })
            }
            placeholder="/opt/homebrew/bin/ffmpeg"
          />
        </Field>
```

**분석 / Analysis (Level 2)**
- API 키 입력 필드가 `<input type="text">` — 마스킹 없음. 어깨너머/스크린쉐어 시 노출 (CWE-549). `type="password"` 가 권장.
- 빈 문자열을 null 로 변환 (`e.target.value || null`) — Settings 옵셔널 타입과 잘 맞춘다.

**짚어볼 점 / Watch-outs**
- **보안**: API 키 평문 표시. type="password" 추가 권장.

```tsx path=src/pages/SettingsPage.tsx lines=127-155
        <div className="rounded-xl border border-line bg-bg-elevated p-4 text-xs text-fg-muted">
          데이터베이스·Whisper 모델·설정은 다음 경로에 저장됩니다:{" "}
          <code className="font-mono text-fg">
            ~/Library/Application Support/com.asitsounds.app
          </code>{" "}
          (macOS 기준).
        </div>
      </div>
    </div>
  );
}

function Field({ label, help, children }: { ... }) {
  return (
    <section className="rounded-xl border border-line bg-bg-elevated p-4">
      <h2 className="mb-1 text-sm font-semibold">{label}</h2>
      {help && <p className="mb-3 text-xs text-fg-muted">{help}</p>}
      {children}
    </section>
  );
}
```

**분석 / Analysis (Level 2)**
- 저장 경로가 하드코딩 — macOS 외에는 부정확하지만 "macOS 기준" 명시.
- `Field` 헬퍼로 마크업 일관성 확보.

**파일 수준 평가 / File-Level Verdict**
- 강점 / Strengths:
  - Field 헬퍼로 UI 일관성 좋음.
  - 옵션 선택 시 default endpoint 자동 갱신 — UX 친절.
  - settings === null 분기로 로딩 명시.
- 위험 / Risks:
  - **보안**: API 키 input type="text" (평문 표시).
  - 매 keystroke 마다 save → 디바운스 없음, 네트워크/저장 부하.
  - settingsGet 실패 처리 부재.
- 개선 권고 / Recommendations:
  - API 키 필드 `type="password"` + 보이기/숨기기 토글.
  - 입력 필드에 300~500ms debounce.
  - settingsGet 에 `.catch(e => toast(...))` 추가.
- 재읽기 권고 레벨 / Suggested re-read level: 2 — 보안/디바운스 패치 후 재검토.
