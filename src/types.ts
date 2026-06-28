export type MediaKind = "audio" | "video";
export type FileStatus = "pending" | "processing" | "done" | "error";

export interface MediaFile {
  id: string;
  projectId: string;
  name: string;
  path: string;
  kind: MediaKind;
  sizeBytes?: number;
  durationMs?: number;
  status: FileStatus;
  error?: string;
  createdAt: number;
}

export interface Segment {
  id: string;
  fileId: string;
  index: number;
  startMs: number;
  endMs: number;
  textEn: string;
  textHangul?: string;
  textKo?: string;
}

export interface Project {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
}

export interface Settings {
  whisperModel: string;
  translateEndpoint: string;
  translateApiKey?: string | null;
  translator: string;
  ffmpegPath?: string | null;
}

export interface NewFileInput {
  name: string;
  path: string;
  kind?: MediaKind;
  sizeBytes?: number;
}

export type PipelineEvent =
  | { type: "fileStarted"; fileId: string }
  | { type: "modelDownloading"; progress: number; message: string }
  | {
      type: "progress";
      fileId: string;
      stage: "decode" | "asr" | "translate" | string;
      message: string;
      percent?: number;
    }
  | { type: "segment"; segment: Segment }
  | { type: "fileDone"; fileId: string; durationMs?: number }
  | { type: "fileError"; fileId: string; error: string }
  | { type: "allDone" };
