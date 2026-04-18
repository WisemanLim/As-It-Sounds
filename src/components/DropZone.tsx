import { useCallback, useState } from "react";
import { IconUpload, IconAudio, IconVideo } from "./icons";
import { cn } from "../lib/cn";
import { detectKind } from "../store/useAppStore";

export interface DroppedFile {
  name: string;
  path: string;
  kind: "audio" | "video";
  sizeBytes?: number;
}

interface DropZoneProps {
  onFiles: (files: DroppedFile[]) => void;
  onPick?: () => void;
  compact?: boolean;
}

const ALLOWED_EXTS = [
  "mp3",
  "wav",
  "m4a",
  "flac",
  "ogg",
  "mp4",
  "mov",
  "mkv",
  "webm",
  "avi",
  "m4v",
];

function isAllowed(name: string) {
  const ext = name.split(".").pop()?.toLowerCase();
  return !!ext && ALLOWED_EXTS.includes(ext);
}

export function DropZone({ onFiles, onPick, compact }: DropZoneProps) {
  const [isOver, setIsOver] = useState(false);

  const handleFiles = useCallback(
    (fileList: FileList | File[]) => {
      const picked: DroppedFile[] = [];
      for (const f of Array.from(fileList)) {
        if (!isAllowed(f.name)) continue;
        const anyFile = f as File & { path?: string };
        picked.push({
          name: f.name,
          path: anyFile.path ?? f.name,
          kind: detectKind(f.name),
          sizeBytes: f.size,
        });
      }
      if (picked.length > 0) onFiles(picked);
    },
    [onFiles],
  );

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
