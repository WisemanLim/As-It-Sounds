import { useEffect, useState } from "react";
import { Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { HomePage } from "./pages/HomePage";
import { ProjectPage } from "./pages/ProjectPage";
import { SettingsPage } from "./pages/SettingsPage";
import { useAppStore } from "./store/useAppStore";
import { PipelineListener } from "./components/PipelineListener";
import { ErrorBoundary } from "./components/ErrorBoundary";

function App() {
  const bootstrap = useAppStore((s) => s.bootstrap);
  const loaded = useAppStore((s) => s.loaded);
  const [bootError, setBootError] = useState<string | null>(null);

  useEffect(() => {
    bootstrap().catch((e) => {
      console.error("bootstrap failed", e);
      setBootError(e instanceof Error ? e.message : String(e));
    });
  }, [bootstrap]);

  if (bootError) {
    return (
      <div className="flex h-screen w-screen flex-col items-center justify-center gap-3 bg-bg p-8 text-center">
        <div className="text-base font-semibold text-red-400">
          초기화에 실패했습니다
        </div>
        <pre className="max-w-2xl whitespace-pre-wrap break-words rounded-lg bg-bg-elevated p-4 text-left font-mono text-xs text-fg-muted">
          {bootError}
        </pre>
        <div className="text-xs text-fg-subtle">
          개발자 도구 콘솔(⌥⌘I)에서 자세한 내용을 확인하세요.
        </div>
      </div>
    );
  }

  if (!loaded) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-bg text-sm text-fg-muted">
        초기화 중...
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <PipelineListener />
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<HomePage />} />
          <Route path="projects/:id" element={<ProjectPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
      </Routes>
    </ErrorBoundary>
  );
}

export default App;
