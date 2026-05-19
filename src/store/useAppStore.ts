import { create } from "zustand";
import { ipc } from "../ipc/client";
import type { PickedMedia } from "../lib/filePicker";
import type {
  FileStatus,
  MediaFile,
  MediaKind,
  Project,
  Segment,
} from "../types";

interface AppState {
  projects: Project[];
  files: MediaFile[];
  segments: Segment[];
  activeProjectId: string | null;
  activeFileId: string | null;
  loaded: boolean;

  bootstrap: () => Promise<void>;

  createProject: (name?: string) => Promise<Project>;
  renameProject: (id: string, name: string) => Promise<void>;
  deleteProject: (id: string) => Promise<void>;
  setActiveProject: (id: string | null) => Promise<void>;

  addFiles: (projectId: string, picked: PickedMedia) => Promise<MediaFile[]>;
  removeFile: (id: string) => Promise<void>;
  setActiveFile: (id: string | null) => Promise<void>;
  resetFile: (id: string) => Promise<void>;

  patchFile: (id: string, patch: Partial<MediaFile>) => void;
  upsertSegment: (segment: Segment) => void;
  clearSegmentsForFile: (fileId: string) => void;
}

export const useAppStore = create<AppState>()((set) => ({
  projects: [],
  files: [],
  segments: [],
  activeProjectId: null,
  activeFileId: null,
  loaded: false,

  async bootstrap() {
    const projects = await ipc.projectList();
    set({ projects, loaded: true });
  },

  async createProject(name) {
    const project = await ipc.projectCreate(name);
    set((s) => ({
      projects: [project, ...s.projects],
      activeProjectId: project.id,
      activeFileId: null,
    }));
    return project;
  },

  async renameProject(id, name) {
    await ipc.projectRename(id, name);
    set((s) => ({
      projects: s.projects.map((p) =>
        p.id === id ? { ...p, name, updatedAt: Date.now() } : p,
      ),
    }));
  },

  async deleteProject(id) {
    await ipc.projectDelete(id);
    set((s) => {
      const fileIds = s.files.filter((f) => f.projectId === id).map((f) => f.id);
      const nextProjects = s.projects.filter((p) => p.id !== id);
      return {
        projects: nextProjects,
        files: s.files.filter((f) => f.projectId !== id),
        segments: s.segments.filter((seg) => !fileIds.includes(seg.fileId)),
        activeProjectId:
          s.activeProjectId === id ? nextProjects[0]?.id ?? null : s.activeProjectId,
        activeFileId: fileIds.includes(s.activeFileId ?? "") ? null : s.activeFileId,
      };
    });
  },

  async setActiveProject(id) {
    set({ activeProjectId: id, activeFileId: null });
    if (!id) {
      set({ files: [], segments: [] });
      return;
    }
    const files = await ipc.fileList(id);
    set((s) => ({
      files: [
        ...s.files.filter((f) => f.projectId !== id),
        ...files,
      ],
    }));

    const segmentsByFile = await Promise.all(
      files.map((f) => ipc.segmentsList(f.id).then((segs) => ({ id: f.id, segs }))),
    );
    set((s) => {
      const keep = s.segments.filter(
        (seg) => !files.some((f) => f.id === seg.fileId),
      );
      const fresh = segmentsByFile.flatMap((r) => r.segs);
      return { segments: [...keep, ...fresh] };
    });
  },

  async addFiles(projectId, picked) {
    let created: MediaFile[];
    if (picked.mode === "paths") {
      if (picked.inputs.length === 0) return [];
      const payload = picked.inputs.map((i) => ({
        name: i.name,
        path: i.path,
        kind: i.kind ?? detectKind(i.name),
        sizeBytes: i.sizeBytes,
      }));
      created = await ipc.fileAddMany(projectId, payload);
    } else {
      if (picked.files.length === 0) return [];
      created = await ipc.fileUpload(projectId, picked.files);
    }
    set((s) => ({
      files: [...s.files, ...created],
      activeFileId: s.activeFileId ?? created[0]?.id ?? null,
      projects: s.projects.map((p) =>
        p.id === projectId ? { ...p, updatedAt: Date.now() } : p,
      ),
    }));
    return created;
  },

  async removeFile(id) {
    await ipc.fileRemove(id);
    set((s) => ({
      files: s.files.filter((f) => f.id !== id),
      segments: s.segments.filter((seg) => seg.fileId !== id),
      activeFileId: s.activeFileId === id ? null : s.activeFileId,
    }));
  },

  async setActiveFile(id) {
    set({ activeFileId: id });
  },

  async resetFile(id) {
    await ipc.fileReset(id);
    set((s) => ({
      files: s.files.map((f) =>
        f.id === id ? { ...f, status: "pending" as FileStatus, error: undefined } : f,
      ),
      segments: s.segments.filter((seg) => seg.fileId !== id),
    }));
  },

  patchFile(id, patch) {
    set((s) => ({
      files: s.files.map((f) => (f.id === id ? { ...f, ...patch } : f)),
    }));
  },

  upsertSegment(segment) {
    set((s) => {
      const others = s.segments.filter(
        (seg) => !(seg.fileId === segment.fileId && seg.index === segment.index),
      );
      return { segments: [...others, segment] };
    });
  },

  clearSegmentsForFile(fileId) {
    set((s) => ({
      segments: s.segments.filter((seg) => seg.fileId !== fileId),
    }));
  },
}));

export function detectKind(name: string): MediaKind {
  const lower = name.toLowerCase();
  const videoExts = [".mp4", ".mov", ".mkv", ".webm", ".avi", ".m4v"];
  if (videoExts.some((ext) => lower.endsWith(ext))) return "video";
  return "audio";
}

