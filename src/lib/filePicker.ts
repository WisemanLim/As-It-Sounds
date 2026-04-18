import { open } from "@tauri-apps/plugin-dialog";
import { stat } from "@tauri-apps/plugin-fs";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { NewFileInput } from "../types";
import { detectKind } from "../store/useAppStore";

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

export async function pickMediaFiles(): Promise<NewFileInput[]> {
  const res = await open({
    multiple: true,
    directory: false,
    filters: [
      {
        name: "Audio & Video",
        extensions: EXTS,
      },
    ],
  });
  if (!res) return [];
  const paths = Array.isArray(res) ? res : [res];
  const out: NewFileInput[] = [];
  for (const path of paths) {
    const p = typeof path === "string" ? path : (path as { path: string }).path;
    let size: number | undefined;
    try {
      const info = await stat(p);
      size = typeof info.size === "number" ? info.size : undefined;
    } catch {
      size = undefined;
    }
    const name = baseName(p);
    out.push({
      name,
      path: p,
      kind: detectKind(name),
      sizeBytes: size,
    });
  }
  return out;
}

export function srcForPath(path: string): string {
  return convertFileSrc(path);
}

function baseName(p: string): string {
  const sep = p.includes("\\") ? "\\" : "/";
  return p.split(sep).pop() || p;
}
