// Runtime detection for Tauri (desktop) vs browser (web).
// The same React bundle ships to both surfaces; ipc / file pickers / drop
// hooks switch on this flag at module load.

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/**
 * Base URL for the web API server. Reads `VITE_AIS_API_BASE` at build time
 * (typically left empty so the frontend hits the same origin behind nginx).
 */
export function apiBaseUrl(): string {
  const env = (import.meta as unknown as { env?: Record<string, string> }).env;
  return env?.VITE_AIS_API_BASE ?? "";
}
