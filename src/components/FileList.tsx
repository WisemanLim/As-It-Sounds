import { cn } from "../lib/cn";
import { formatDuration } from "../lib/format";
import { useAppStore } from "../store/useAppStore";
import type { MediaFile } from "../types";
import { IconAudio, IconTrash, IconVideo } from "./icons";

interface Stage {
  stage: string;
  message: string;
}

interface Props {
  files: MediaFile[];
  stages?: Record<string, Stage>;
}

const statusStyle: Record<MediaFile["status"], string> = {
  pending: "bg-bg-hover text-fg-muted",
  processing: "bg-brand-soft text-brand",
  done: "bg-emerald-950 text-emerald-300",
  error: "bg-red-950 text-red-300",
};

const statusLabel: Record<MediaFile["status"], string> = {
  pending: "대기",
  processing: "처리 중",
  done: "완료",
  error: "오류",
};

export function FileList({ files, stages }: Props) {
  const activeFileId = useAppStore((s) => s.activeFileId);
  const setActiveFile = useAppStore((s) => s.setActiveFile);
  const removeFile = useAppStore((s) => s.removeFile);

  if (files.length === 0) {
    return null;
  }

  return (
    <ul className="flex flex-col gap-1">
      {files.map((f) => {
        const isActive = f.id === activeFileId;
        const Icon = f.kind === "video" ? IconVideo : IconAudio;
        const stage = stages?.[f.id];
        return (
          <li key={f.id}>
            <div
              onClick={() => setActiveFile(f.id)}
              className={cn(
                "group flex cursor-pointer items-center gap-3 rounded-xl border px-3 py-2.5 transition-colors",
                isActive
                  ? "border-brand/50 bg-brand-soft/30"
                  : "border-line bg-bg-elevated hover:bg-bg-hover",
              )}
              title={f.path}
            >
              <div
                className={cn(
                  "flex h-9 w-9 shrink-0 items-center justify-center rounded-lg",
                  f.kind === "video"
                    ? "bg-fuchsia-950/40 text-fuchsia-300"
                    : "bg-brand-soft text-brand",
                )}
              >
                <Icon width={18} height={18} />
              </div>

              <div className="min-w-0 flex-1">
                <div className="truncate text-sm font-medium text-fg">
                  {f.name}
                </div>
                <div className="mt-0.5 flex items-center gap-2 text-xs text-fg-subtle">
                  <span>{formatDuration(f.durationMs)}</span>
                  {f.sizeBytes != null && (
                    <>
                      <span>·</span>
                      <span>{(f.sizeBytes / (1024 * 1024)).toFixed(1)} MB</span>
                    </>
                  )}
                  {stage && (
                    <>
                      <span>·</span>
                      <span className="truncate text-brand">{stage.message}</span>
                    </>
                  )}
                </div>
                {f.error && (
                  <div className="mt-1 truncate text-xs text-red-400" title={f.error}>
                    {f.error}
                  </div>
                )}
              </div>

              <span
                className={cn(
                  "rounded-full px-2 py-0.5 text-[11px] font-medium",
                  statusStyle[f.status],
                )}
              >
                {statusLabel[f.status]}
              </span>

              <button
                className="invisible rounded p-1.5 text-fg-subtle hover:bg-bg hover:text-fg group-hover:visible"
                onClick={(e) => {
                  e.stopPropagation();
                  removeFile(f.id);
                }}
                title="파일 제거"
              >
                <IconTrash width={14} height={14} />
              </button>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
