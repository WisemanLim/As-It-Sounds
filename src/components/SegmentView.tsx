import { cn } from "../lib/cn";
import { formatTime } from "../lib/format";
import type { Segment } from "../types";

interface Props {
  segments: Segment[];
  currentTimeMs: number;
  onSeek?: (ms: number) => void;
}

export function SegmentView({ segments, currentTimeMs, onSeek }: Props) {
  if (segments.length === 0) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-fg-muted">
        아직 변환된 세그먼트가 없습니다. "변환 시작"을 눌러 시작하세요.
      </div>
    );
  }

  return (
    <ol className="flex flex-col gap-2">
      {segments.map((seg) => {
        const active =
          currentTimeMs >= seg.startMs && currentTimeMs < seg.endMs;
        return (
          <li key={seg.id}>
            <button
              onClick={() => onSeek?.(seg.startMs)}
              className={cn(
                "w-full rounded-xl border px-4 py-3 text-left transition-colors",
                active
                  ? "border-brand/60 bg-brand-soft/30"
                  : "border-line bg-bg-elevated hover:bg-bg-hover",
              )}
            >
              <div className="flex items-center justify-between text-[11px] font-mono text-fg-subtle">
                <span>
                  {formatTime(seg.startMs)} — {formatTime(seg.endMs)}
                </span>
                <span>#{seg.index + 1}</span>
              </div>

              <div className="mt-2 space-y-1.5">
                <p className="text-sm leading-relaxed text-fg">
                  <span className="mr-2 inline-block rounded bg-bg-hover px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-fg-subtle">
                    EN
                  </span>
                  {seg.textEn}
                </p>
                {seg.textHangul && (
                  <p className="text-sm leading-relaxed text-brand">
                    <span className="mr-2 inline-block rounded bg-brand-soft px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-brand">
                      발음
                    </span>
                    {seg.textHangul}
                  </p>
                )}
                {seg.textKo && (
                  <p className="text-sm leading-relaxed text-fg-muted">
                    <span className="mr-2 inline-block rounded bg-bg-hover px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-fg-subtle">
                      번역
                    </span>
                    {seg.textKo}
                  </p>
                )}
              </div>
            </button>
          </li>
        );
      })}
    </ol>
  );
}
