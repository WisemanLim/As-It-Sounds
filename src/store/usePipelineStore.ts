import { create } from "zustand";

interface Stage {
  stage: string;
  message: string;
  percent: number | null;
}

interface Download {
  progress: number;
  message: string;
}

interface PipelineState {
  stages: Record<string, Stage>;
  download: Download | null;
  setStage: (fileId: string, stage: Stage) => void;
  setDone: (fileId: string) => void;
  setDownload: (d: Download | null) => void;
  clear: () => void;
}

export const usePipelineStore = create<PipelineState>()((set) => ({
  stages: {},
  download: null,
  setStage: (fileId, stage) =>
    set((s) => ({ stages: { ...s.stages, [fileId]: stage } })),
  setDone: (fileId) =>
    set((s) => {
      const next = { ...s.stages };
      delete next[fileId];
      return { stages: next };
    }),
  setDownload: (d) => set({ download: d }),
  clear: () => set({ stages: {}, download: null }),
}));
