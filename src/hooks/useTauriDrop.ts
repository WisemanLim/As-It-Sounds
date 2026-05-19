// Window-level file drop handler for the Tauri webview.
// On web this hook is a no-op — the browser `DropZone` element handles HTML5
// drag/drop directly there.

import { useEffect, useRef } from "react";
import type { NewFileInput } from "../types";
import { detectKind } from "../store/useAppStore";
import { isTauri } from "../lib/env";
import type { PickedMedia } from "../lib/filePicker";

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

async function tauriPathsToInputs(paths: string[]): Promise<NewFileInput[]> {
  const { stat } = await import("@tauri-apps/plugin-fs");
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
  onDrop: (picked: PickedMedia) => void,
  onHover?: (hovering: boolean) => void,
) {
  const onDropRef = useRef(onDrop);
  const onHoverRef = useRef(onHover);

  useEffect(() => {
    onDropRef.current = onDrop;
  }, [onDrop]);
  useEffect(() => {
    onHoverRef.current = onHover;
  }, [onHover]);

  useEffect(() => {
    if (!isTauri) return;
    let disposed = false;
    let unlisten: (() => void) | null = null;
    const safeUnlisten = (fn: (() => void) | null) => {
      if (!fn) return;
      try {
        fn();
      } catch (e) {
        console.debug("drop listener already unregistered", e);
      }
    };
    const setup = async () => {
      try {
        const { getCurrentWebview } = await import("@tauri-apps/api/webview");
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
            const inputs = await tauriPathsToInputs(payload.paths ?? []);
            if (inputs.length > 0)
              onDropRef.current({ mode: "paths", inputs });
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
