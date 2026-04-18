import { useEffect, useRef } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { stat } from "@tauri-apps/plugin-fs";
import type { NewFileInput } from "../types";
import { detectKind } from "../store/useAppStore";

const EXTS = new Set([
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
]);

function baseName(p: string): string {
  const sep = p.includes("\\") ? "\\" : "/";
  return p.split(sep).pop() || p;
}

async function toInputs(paths: string[]): Promise<NewFileInput[]> {
  const out: NewFileInput[] = [];
  for (const p of paths) {
    const name = baseName(p);
    const ext = name.split(".").pop()?.toLowerCase();
    if (!ext || !EXTS.has(ext)) continue;
    let size: number | undefined;
    try {
      const info = await stat(p);
      size = typeof info.size === "number" ? info.size : undefined;
    } catch {
      size = undefined;
    }
    out.push({ name, path: p, kind: detectKind(name), sizeBytes: size });
  }
  return out;
}

export function useTauriDrop(
  onFiles: (inputs: NewFileInput[]) => void,
  onHover?: (hovering: boolean) => void,
) {
  const onFilesRef = useRef(onFiles);
  const onHoverRef = useRef(onHover);

  useEffect(() => {
    onFilesRef.current = onFiles;
  }, [onFiles]);
  useEffect(() => {
    onHoverRef.current = onHover;
  }, [onHover]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;
    const safeUnlisten = (fn: (() => void) | null) => {
      if (!fn) return;
      try {
        fn();
      } catch (e) {
        // Tauri event registry throws if the listener was already removed
        // (e.g. under StrictMode double-mount); swallowing is intentional.
        console.debug("drop listener already unregistered", e);
      }
    };
    const setup = async () => {
      try {
        const u = await getCurrentWebview().onDragDropEvent(async (event) => {
          if (disposed) return;
          const payload = event.payload as unknown as {
            type?: string;
            paths?: string[];
          };
          if (payload.type === "enter" || payload.type === "over") {
            onHoverRef.current?.(true);
          } else if (payload.type === "leave" || payload.type === "cancel") {
            onHoverRef.current?.(false);
          } else if (payload.type === "drop") {
            onHoverRef.current?.(false);
            const inputs = await toInputs(payload.paths ?? []);
            if (inputs.length > 0) onFilesRef.current(inputs);
          }
        });
        if (disposed) {
          safeUnlisten(u);
          return;
        }
        unlisten = u;
      } catch (e) {
        console.warn("native drag/drop unavailable", e);
      }
    };
    setup();
    return () => {
      disposed = true;
      safeUnlisten(unlisten);
      unlisten = null;
    };
  }, []);
}
