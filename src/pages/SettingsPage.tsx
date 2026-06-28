import { useEffect, useState } from "react";
import { ipc } from "../ipc/client";
import type { Settings } from "../types";
import toast from "../lib/toast";

const MODEL_OPTIONS: Array<{ value: string; label: string }> = [
  { value: "tiny.en", label: "tiny.en (75 MB · 가장 빠름 · 영어 전용)" },
  { value: "tiny", label: "tiny (75 MB · 가장 빠름 · 다국어)" },
  { value: "base.en", label: "base.en (142 MB · 기본 권장 · 영어 전용)" },
  { value: "base", label: "base (142 MB · 기본 권장 · 다국어)" },
  { value: "small.en", label: "small.en (466 MB · 균형 · 영어 전용)" },
  { value: "small", label: "small (466 MB · 균형 · 다국어)" },
  { value: "medium.en", label: "medium.en (1.5 GB · 정확 · 영어 전용)" },
  { value: "medium", label: "medium (1.5 GB · 정확 · 다국어)" },
  { value: "large-v3-turbo", label: "large-v3-turbo (1.6 GB · 빠른 대형 · 다국어)" },
  { value: "large-v3", label: "large-v3 (2.9 GB · 최고 품질 · 다국어)" },
];

const TRANSLATOR_OPTIONS: Array<{ value: string; label: string; endpoint: string }> = [
  {
    value: "mymemory",
    label: "MyMemory (무료, 키 불필요, 1000단어/일)",
    endpoint: "https://api.mymemory.translated.net/get",
  },
  {
    value: "libretranslate",
    label: "LibreTranslate (셀프호스트)",
    endpoint: "https://libretranslate.com/translate",
  },
  { value: "none", label: "번역 사용 안 함", endpoint: "" },
];

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

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-line px-6">
        <h1 className="text-sm font-semibold">설정</h1>
        {saving && <span className="text-xs text-fg-muted">저장 중...</span>}
      </header>
      <div className="mx-auto w-full max-w-2xl px-6 py-8 space-y-6 text-sm">
        <Field label="Whisper 모델" help="첫 사용 시 자동 다운로드됩니다.">
          <select
            className="w-full rounded-lg border border-line bg-bg px-3 py-2"
            value={settings.whisperModel}
            onChange={(e) => save({ ...settings, whisperModel: e.target.value })}
          >
            {MODEL_OPTIONS.map((m) => (
              <option key={m.value} value={m.value}>
                {m.label}
              </option>
            ))}
          </select>
        </Field>

        <Field label="번역 엔진">
          <select
            className="w-full rounded-lg border border-line bg-bg px-3 py-2"
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
            {TRANSLATOR_OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
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
            className="w-full rounded-lg border border-line bg-bg px-3 py-2 font-mono text-xs"
            value={settings.ffmpegPath ?? ""}
            onChange={(e) =>
              save({ ...settings, ffmpegPath: e.target.value || null })
            }
            placeholder="/opt/homebrew/bin/ffmpeg"
          />
        </Field>

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

function Field({
  label,
  help,
  children,
}: {
  label: string;
  help?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-xl border border-line bg-bg-elevated p-4">
      <h2 className="mb-1 text-sm font-semibold">{label}</h2>
      {help && <p className="mb-3 text-xs text-fg-muted">{help}</p>}
      {children}
    </section>
  );
}
