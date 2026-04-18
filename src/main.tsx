import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter } from "react-router-dom";
import App from "./App";
import "./index.css";

declare global {
  interface Window {
    __fatalShown?: boolean;
  }
}

function showFatal(message: string, stack?: string) {
  if (window.__fatalShown) return;
  window.__fatalShown = true;
  const overlay = document.createElement("div");
  overlay.style.cssText =
    "position:fixed;inset:0;z-index:2147483647;background:#0e0f11;color:#e6e7ea;" +
    "font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif;" +
    "padding:32px;display:flex;flex-direction:column;gap:12px;" +
    "align-items:center;justify-content:center;text-align:center;";
  const title = document.createElement("div");
  title.textContent = "오류가 발생했습니다";
  title.style.cssText = "color:#fca5a5;font-weight:600;";
  const pre = document.createElement("pre");
  pre.textContent = `${message}\n\n${stack ?? ""}`;
  pre.style.cssText =
    "max-width:720px;white-space:pre-wrap;word-break:break-word;" +
    "background:#17191c;padding:16px;border-radius:8px;" +
    "font-family:ui-monospace,Menlo,monospace;font-size:12px;color:#9aa0a6;text-align:left;";
  overlay.appendChild(title);
  overlay.appendChild(pre);
  document.body.appendChild(overlay);
}

window.addEventListener("unhandledrejection", (e) => {
  const reason = e.reason as { message?: string; stack?: string } | undefined;
  showFatal(reason?.message ?? String(reason), reason?.stack);
});

try {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <HashRouter>
        <App />
      </HashRouter>
    </React.StrictMode>,
  );
} catch (e) {
  showFatal(
    e instanceof Error ? e.message : String(e),
    e instanceof Error ? e.stack : undefined,
  );
}
