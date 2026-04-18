import { useEffect } from "react";
import toast from "../lib/toast";
import { onPipelineEvent } from "../ipc/client";
import { usePipelineStore } from "../store/usePipelineStore";
import { useAppStore } from "../store/useAppStore";

export function PipelineListener() {
  const setStage = usePipelineStore((s) => s.setStage);
  const setDownload = usePipelineStore((s) => s.setDownload);
  const setDone = usePipelineStore((s) => s.setDone);

  const patchFile = useAppStore((s) => s.patchFile);
  const upsertSegment = useAppStore((s) => s.upsertSegment);
  const clearSegmentsForFile = useAppStore((s) => s.clearSegmentsForFile);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    onPipelineEvent((e) => {
      switch (e.type) {
        case "fileStarted":
          patchFile(e.fileId, { status: "processing", error: undefined });
          clearSegmentsForFile(e.fileId);
          setStage(e.fileId, { stage: "decode", message: "시작..." });
          break;
        case "modelDownloading":
          setDownload({ progress: e.progress, message: e.message });
          break;
        case "progress":
          setStage(e.fileId, { stage: e.stage, message: e.message });
          break;
        case "segment":
          upsertSegment(e.segment);
          break;
        case "fileDone":
          patchFile(e.fileId, {
            status: "done",
            durationMs: e.durationMs ?? undefined,
          });
          setDone(e.fileId);
          break;
        case "fileError":
          patchFile(e.fileId, { status: "error", error: e.error });
          setDone(e.fileId);
          toast(`변환 실패: ${e.error}`);
          break;
        case "allDone":
          setDownload(null);
          break;
      }
    }).then((u) => {
      unlisten = u;
    });
    return () => {
      unlisten?.();
    };
  }, [setStage, setDownload, setDone, patchFile, upsertSegment, clearSegmentsForFile]);

  return null;
}
