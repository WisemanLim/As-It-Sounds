// IPC switcher — picks the desktop (Tauri) or web (HTTP/SSE) client at runtime.
// Components import `ipc` and `onPipelineEvent` here without caring which
// surface they're running on.

import type { PipelineEvent } from "../types";
import { isTauri } from "../lib/env";
import { tauriIpc, tauriOnPipelineEvent } from "./tauriClient";
import { webIpc, webOnPipelineEvent } from "./webClient";

export const ipc = isTauri ? tauriIpc : webIpc;

export function onPipelineEvent(
  handler: (e: PipelineEvent) => void,
): Promise<() => void> {
  return isTauri ? tauriOnPipelineEvent(handler) : webOnPipelineEvent(handler);
}
