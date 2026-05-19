import { useNavigate } from "react-router-dom";
import { DropZone } from "../components/DropZone";
import { useAppStore } from "../store/useAppStore";
import { IconPlus, IconSparkles } from "../components/icons";
import { pickMediaFiles, type PickedMedia } from "../lib/filePicker";
import { useTauriDrop } from "../hooks/useTauriDrop";
import toast from "../lib/toast";

export function HomePage() {
  const navigate = useNavigate();
  const createProject = useAppStore((s) => s.createProject);
  const addFiles = useAppStore((s) => s.addFiles);

  const handleCreate = async () => {
    try {
      const p = await createProject();
      navigate(`/projects/${p.id}`);
    } catch (e) {
      console.error("createProject failed", e);
      toast(`프로젝트 생성 실패: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const startWithPicked = async (picked: PickedMedia) => {
    const isEmpty =
      picked.mode === "paths" ? picked.inputs.length === 0 : picked.files.length === 0;
    if (isEmpty) return;
    const p = await createProject();
    await addFiles(p.id, picked);
    navigate(`/projects/${p.id}`);
  };

  const handlePick = async () => {
    try {
      const picked = await pickMediaFiles();
      await startWithPicked(picked);
    } catch (e) {
      console.error("handlePick failed", e);
      toast(`파일 추가 실패: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleDropZoneFiles = async (picked: PickedMedia) => {
    try {
      await startWithPicked(picked);
    } catch (e) {
      console.error("dropzone files failed", e);
      toast(`파일 추가 실패: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  useTauriDrop(async (picked) => {
    try {
      await startWithPicked(picked);
    } catch (e) {
      console.error("drop create project failed", e);
    }
  });

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-line px-6">
        <h1 className="text-sm font-semibold text-fg-muted">As It Sounds</h1>
      </header>

      <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center gap-8 px-6 py-10">
        <div className="space-y-2 text-center">
          <div className="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-2xl bg-brand-soft text-brand">
            <IconSparkles />
          </div>
          <h2 className="text-2xl font-semibold tracking-tight">
            영어 음성을 한글 발음 그대로 볼까요?
          </h2>
          <p className="text-sm text-fg-muted">
            새 프로젝트를 만들거나 파일을 바로 끌어다 놓아 시작하세요.
          </p>
        </div>

        <DropZone onFiles={handleDropZoneFiles} onPick={handlePick} />

        <div className="flex justify-center">
          <button className="btn-outline" onClick={handleCreate}>
            <IconPlus />
            빈 프로젝트로 시작
          </button>
        </div>
      </div>
    </div>
  );
}
