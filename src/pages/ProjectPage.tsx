import { useEffect, useMemo, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { DropZone } from "../components/DropZone";
import { FileList } from "../components/FileList";
import { MediaPlayer, type MediaPlayerHandle } from "../components/MediaPlayer";
import { SegmentView } from "../components/SegmentView";
import { IconPlus, IconSparkles } from "../components/icons";
import { useAppStore } from "../store/useAppStore";
import { usePipelineStore } from "../store/usePipelineStore";
import { ipc } from "../ipc/client";
import { pickMediaFiles } from "../lib/filePicker";
import { useTauriDrop } from "../hooks/useTauriDrop";

export function ProjectPage() {
  const { id } = useParams<{ id: string }>();

  // Only select stable top-level state from the store; derive locally.
  const projects = useAppStore((s) => s.projects);
  const allFiles = useAppStore((s) => s.files);
  const allSegments = useAppStore((s) => s.segments);
  const activeFileId = useAppStore((s) => s.activeFileId);

  const addFiles = useAppStore((s) => s.addFiles);
  const setActiveFile = useAppStore((s) => s.setActiveFile);
  const setActiveProject = useAppStore((s) => s.setActiveProject);
  const renameProject = useAppStore((s) => s.renameProject);

  const stages = usePipelineStore((s) => s.stages);
  const download = usePipelineStore((s) => s.download);

  const project = useMemo(
    () => projects.find((p) => p.id === id) ?? null,
    [projects, id],
  );
  const files = useMemo(
    () => allFiles.filter((f) => f.projectId === id),
    [allFiles, id],
  );
  const sortedFiles = useMemo(
    () => [...files].sort((a, b) => a.createdAt - b.createdAt),
    [files],
  );
  const activeFile = useMemo(
    () =>
      allFiles.find((f) => f.id === activeFileId && f.projectId === id) ?? null,
    [allFiles, activeFileId, id],
  );
  const segments = useMemo(
    () =>
      allSegments
        .filter((seg) => seg.fileId === activeFile?.id)
        .slice()
        .sort((a, b) => a.index - b.index),
    [allSegments, activeFile?.id],
  );

  const [currentTimeMs, setCurrentTimeMs] = useState(0);
  const playerRef = useRef<MediaPlayerHandle>(null);

  useEffect(() => {
    if (!id) return;
    setActiveProject(id).catch((e) => {
      console.error("setActiveProject failed", e);
    });
  }, [id, setActiveProject]);

  useTauriDrop(async (picked) => {
    if (!id) return;
    try {
      const created = await addFiles(id, picked);
      if (created.length > 0) await setActiveFile(created[0].id);
    } catch (e) {
      console.error("drop add files failed", e);
    }
  });

  if (!project) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-fg-muted">
        프로젝트를 찾을 수 없습니다.
      </div>
    );
  }

  const handlePick = async () => {
    try {
      const picked = await pickMediaFiles();
      const created = await addFiles(project.id, picked);
      if (created.length > 0 && !activeFileId) {
        await setActiveFile(created[0].id);
      }
    } catch (e) {
      console.error("handlePick failed", e);
    }
  };

  const handleDropZoneFiles = async (picked: import("../lib/filePicker").PickedMedia) => {
    try {
      const created = await addFiles(project.id, picked);
      if (created.length > 0 && !activeFileId) {
        await setActiveFile(created[0].id);
      }
    } catch (e) {
      console.error("handleDropZoneFiles failed", e);
    }
  };

  const runPipeline = async () => {
    try {
      const ids = sortedFiles
        .filter((f) => f.status !== "processing")
        .map((f) => f.id);
      if (ids.length === 0) return;
      await ipc.pipelineRun(ids);
    } catch (e) {
      console.error("pipelineRun failed", e);
    }
  };

  const mediaSrc = activeFile ? ipc.mediaSrc(activeFile) : null;

  const hasFiles = sortedFiles.length > 0;
  const anyProcessing = sortedFiles.some((f) => f.status === "processing");
  const activeStage = activeFile ? stages[activeFile.id] : null;

  return (
    <div className="flex h-full flex-col">
      <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b border-line px-6">
        <input
          className="min-w-0 flex-1 truncate bg-transparent text-sm font-semibold text-fg placeholder:text-fg-subtle focus:outline-none"
          value={project.name}
          onChange={(e) =>
            renameProject(project.id, e.target.value).catch((err) =>
              console.error("rename failed", err),
            )
          }
        />
        <div className="flex items-center gap-2">
          <button className="btn-outline" onClick={handlePick}>
            <IconPlus />
            파일 추가
          </button>
          <button
            className="btn-primary"
            onClick={runPipeline}
            disabled={!hasFiles || anyProcessing}
          >
            <IconSparkles />
            {anyProcessing ? "처리 중..." : "변환 시작"}
          </button>
        </div>
      </header>

      {download && (
        <div className="border-b border-line bg-brand-soft/30 px-6 py-2 text-xs text-brand">
          {download.message}{" "}
          {download.progress > 0 &&
            `· ${Math.round(download.progress * 100)}%`}
          <div className="mt-1 h-1 w-full overflow-hidden rounded bg-bg-hover">
            <div
              className="h-full bg-brand transition-all"
              style={{ width: `${Math.round(download.progress * 100)}%` }}
            />
          </div>
        </div>
      )}

      {!hasFiles ? (
        <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center px-6 py-10">
          <DropZone onFiles={handleDropZoneFiles} onPick={handlePick} />
        </div>
      ) : (
        <div className="grid min-h-0 flex-1 grid-cols-[minmax(280px,340px)_1fr] gap-4 overflow-hidden p-4">
          <aside className="flex min-h-0 flex-col gap-3 overflow-y-auto pr-1">
            <DropZone onFiles={handlePick} onPick={handlePick} compact />
            <FileList files={sortedFiles} stages={stages} />
          </aside>

          <section className="flex min-h-0 flex-col gap-3 overflow-hidden">
            {activeFile && mediaSrc ? (
              <MediaPlayer
                ref={playerRef}
                file={activeFile}
                src={mediaSrc}
                onTimeUpdate={setCurrentTimeMs}
                onDurationChange={() => {
                  /* backend sets duration during pipeline */
                }}
              />
            ) : (
              <div className="rounded-xl border border-dashed border-line bg-bg-elevated px-4 py-6 text-sm text-fg-muted">
                좌측에서 파일을 선택하세요.
              </div>
            )}

            {activeStage && (
              <div className="rounded-lg border border-line bg-bg-elevated px-3 py-2 text-xs text-fg-muted">
                <span className="mr-2 rounded bg-brand-soft px-1.5 py-0.5 font-medium text-brand">
                  {activeStage.stage}
                </span>
                {activeStage.message}
              </div>
            )}

            <div className="min-h-0 flex-1 overflow-y-auto pr-1">
              <SegmentView
                segments={segments}
                currentTimeMs={currentTimeMs}
                onSeek={(ms) => playerRef.current?.seek(ms)}
              />
            </div>
          </section>
        </div>
      )}
    </div>
  );
}
