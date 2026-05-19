// File pickers — mode-aware. On desktop opens the Tauri dialog (returns absolute
// paths the Rust backend can read); on web opens a hidden `<input type="file">`
// (returns browser File objects that the caller uploads via multipart).

import type { NewFileInput } from "../types";
import { detectKind } from "../store/useAppStore";
import { isTauri } from "./env";

const EXTS = [
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

export type PickedMedia =
  | { mode: "paths"; inputs: NewFileInput[] }
  | { mode: "files"; files: File[] };

function baseName(p: string): string {
  const sep = p.includes("\\") ? "\\" : "/";
  return p.split(sep).pop() || p;
}

async function pickViaTauri(): Promise<PickedMedia> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const { stat } = await import("@tauri-apps/plugin-fs");
  const res = await open({
    multiple: true,
    directory: false,
    filters: [{ name: "Audio & Video", extensions: EXTS }],
  });
  if (!res) return { mode: "paths", inputs: [] };
  const paths = Array.isArray(res) ? res : [res];
  const out: NewFileInput[] = [];
  for (const path of paths) {
    const p =
      typeof path === "string" ? path : (path as { path: string }).path;
    let size: number | undefined;
    try {
      const info = await stat(p);
      size = typeof info.size === "number" ? info.size : undefined;
    } catch {
      size = undefined;
    }
    const name = baseName(p);
    out.push({ name, path: p, kind: detectKind(name), sizeBytes: size });
  }
  return { mode: "paths", inputs: out };
}

function pickViaBrowser(): Promise<PickedMedia> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.multiple = true;
    input.accept = EXTS.map((e) => `.${e}`).join(",");
    // Some browsers garbage-collect detached inputs before `change` fires; keep
    // a reference until the picker resolves.
    document.body.appendChild(input);
    input.addEventListener(
      "change",
      () => {
        const files = input.files ? Array.from(input.files) : [];
        input.remove();
        resolve({ mode: "files", files });
      },
      { once: true },
    );
    // Edge case: user cancels — no `change` event fires. We get a `cancel`
    // event in modern browsers; fall back to a focus listener for the rest.
    input.addEventListener(
      "cancel",
      () => {
        input.remove();
        resolve({ mode: "files", files: [] });
      },
      { once: true },
    );
    input.click();
  });
}

export function pickMediaFiles(): Promise<PickedMedia> {
  return isTauri ? pickViaTauri() : pickViaBrowser();
}

export function isAllowedMediaName(name: string): boolean {
  const ext = name.split(".").pop()?.toLowerCase();
  return !!ext && EXTS.includes(ext);
}
