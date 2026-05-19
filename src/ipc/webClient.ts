// Web (browser) IPC client — talks to the axum server via REST + SSE.
// Surface matches `tauriIpc` so the rest of the app stays oblivious.

import type {
  MediaFile,
  NewFileInput,
  PipelineEvent,
  Project,
  Segment,
  Settings,
} from "../types";
import { apiBaseUrl } from "../lib/env";

function base(): string {
  return apiBaseUrl();
}

async function jget<T>(path: string): Promise<T> {
  const r = await fetch(`${base()}/api${path}`);
  if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
  return (await r.json()) as T;
}

async function jsend<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T | void> {
  const r = await fetch(`${base()}/api${path}`, {
    method,
    headers: body !== undefined ? { "content-type": "application/json" } : {},
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
  if (r.status === 204) return;
  const ct = r.headers.get("content-type") ?? "";
  if (!ct.includes("application/json")) return;
  return (await r.json()) as T;
}

export const webIpc = {
  projectList: () => jget<Project[]>("/projects"),
  projectCreate: (name?: string) =>
    jsend<Project>("POST", "/projects", { name: name ?? null }) as Promise<Project>,
  projectRename: async (id: string, name: string): Promise<void> => {
    await jsend("PATCH", `/projects/${encodeURIComponent(id)}`, { name });
  },
  projectDelete: async (id: string): Promise<void> => {
    await jsend("DELETE", `/projects/${encodeURIComponent(id)}`);
  },

  fileList: (projectId: string) =>
    jget<MediaFile[]>(`/projects/${encodeURIComponent(projectId)}/files`),

  fileAddMany: async (
    _projectId: string,
    _inputs: NewFileInput[],
  ): Promise<MediaFile[]> => {
    throw new Error("fileAddMany is desktop-only — web uses fileUpload");
  },

  fileUpload: async (projectId: string, files: File[]): Promise<MediaFile[]> => {
    const form = new FormData();
    for (const f of files) form.append("files", f, f.name);
    const r = await fetch(
      `${base()}/api/projects/${encodeURIComponent(projectId)}/files`,
      { method: "POST", body: form },
    );
    if (!r.ok) throw new Error(await r.text().catch(() => r.statusText));
    return (await r.json()) as MediaFile[];
  },

  fileRemove: async (id: string): Promise<void> => {
    await jsend("DELETE", `/files/${encodeURIComponent(id)}`);
  },

  fileReset: async (id: string): Promise<void> => {
    await jsend("POST", `/files/${encodeURIComponent(id)}/reset`);
  },

  segmentsList: (fileId: string) =>
    jget<Segment[]>(`/files/${encodeURIComponent(fileId)}/segments`),

  settingsGet: () => jget<Settings>("/settings"),
  settingsSet: (settings: Settings) =>
    jsend<Settings>("PUT", "/settings", settings) as Promise<Settings>,

  pipelineRun: async (fileIds: string[]): Promise<void> => {
    await jsend("POST", "/pipeline/run", { fileIds });
  },

  translitPreview: async (text: string): Promise<string> => {
    const out = (await jsend<{ result: string }>("POST", "/translit/preview", {
      text,
    })) as { result: string };
    return out.result;
  },

  mediaSrc: (file: MediaFile): string =>
    `${base()}/api/files/${encodeURIComponent(file.id)}/media`,
};

export function webOnPipelineEvent(
  handler: (e: PipelineEvent) => void,
): Promise<() => void> {
  // EventSource auto-reconnects on transient drops; we expose a single sync
  // disposer for parity with Tauri's `UnlistenFn`.
  const es = new EventSource(`${base()}/api/events`);
  es.onmessage = (ev) => {
    try {
      handler(JSON.parse(ev.data) as PipelineEvent);
    } catch (e) {
      console.warn("bad SSE payload", e);
    }
  };
  es.onerror = () => {
    // Browser will auto-reconnect; surface only persistent failures via console.
    console.debug("SSE transient error");
  };
  return Promise.resolve(() => es.close());
}
