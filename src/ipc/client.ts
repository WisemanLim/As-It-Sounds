import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  MediaFile,
  Project,
  Segment,
  Settings,
  PipelineEvent,
  NewFileInput,
} from "../types";

export const ipc = {
  projectList: () => invoke<Project[]>("project_list"),
  projectCreate: (name?: string) =>
    invoke<Project>("project_create", { name: name ?? null }),
  projectRename: (id: string, name: string) =>
    invoke<void>("project_rename", { id, name }),
  projectDelete: (id: string) => invoke<void>("project_delete", { id }),

  fileList: (projectId: string) =>
    invoke<MediaFile[]>("file_list", { projectId }),
  fileAddMany: (projectId: string, inputs: NewFileInput[]) =>
    invoke<MediaFile[]>("file_add_many", { projectId, inputs }),
  fileRemove: (id: string) => invoke<void>("file_remove", { id }),
  fileReset: (id: string) => invoke<void>("file_reset", { id }),

  segmentsList: (fileId: string) =>
    invoke<Segment[]>("segments_list", { fileId }),

  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (settings: Settings) =>
    invoke<Settings>("settings_set", { settings }),

  pipelineRun: (fileIds: string[]) =>
    invoke<void>("pipeline_run", { fileIds }),

  translitPreview: (text: string) =>
    invoke<string>("translit_preview", { text }),
};

export function onPipelineEvent(
  handler: (e: PipelineEvent) => void,
): Promise<UnlistenFn> {
  return listen<PipelineEvent>("pipeline://event", (event) =>
    handler(event.payload),
  );
}
