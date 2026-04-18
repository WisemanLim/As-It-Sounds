import { useMemo, useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { confirm as tauriConfirm } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "../store/useAppStore";
import {
  IconMenu,
  IconPlus,
  IconSearch,
  IconSettings,
  IconSparkles,
  IconTrash,
} from "./icons";
import { cn } from "../lib/cn";

export function Sidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const [collapsed, setCollapsed] = useState(false);
  const [query, setQuery] = useState("");

  const projects = useAppStore((s) => s.projects);
  const activeProjectId = useAppStore((s) => s.activeProjectId);
  const createProject = useAppStore((s) => s.createProject);
  const deleteProject = useAppStore((s) => s.deleteProject);
  const setActiveProject = useAppStore((s) => s.setActiveProject);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return projects;
    return projects.filter((p) => p.name.toLowerCase().includes(q));
  }, [projects, query]);

  const handleNewProject = () => {
    const id = createProject();
    navigate(`/projects/${id}`);
  };

  const handleOpenProject = (id: string) => {
    setActiveProject(id);
    navigate(`/projects/${id}`);
  };

  return (
    <aside
      className={cn(
        "flex h-full flex-col border-r border-line bg-bg-sidebar transition-[width] duration-200",
        collapsed ? "w-16" : "w-[260px]",
      )}
    >
      <div className="flex items-center gap-2 px-3 py-3">
        <button
          className="btn-ghost h-9 w-9 p-0"
          onClick={() => setCollapsed((c) => !c)}
          title="사이드바 토글"
        >
          <IconMenu />
        </button>
        {!collapsed && (
          <div className="flex items-center gap-2">
            <IconSparkles className="text-brand" />
            <span className="text-sm font-semibold tracking-tight">As It Sounds</span>
          </div>
        )}
      </div>

      <div className="px-3">
        <button
          onClick={handleNewProject}
          className={cn(
            "btn-outline w-full",
            collapsed && "justify-center px-0",
          )}
        >
          <IconPlus />
          {!collapsed && <span>새 프로젝트</span>}
        </button>
      </div>

      {!collapsed && (
        <div className="mt-3 px-3">
          <div className="flex items-center gap-2 rounded-lg border border-line bg-bg-elevated px-2 py-1.5 text-sm">
            <IconSearch className="text-fg-subtle" width={16} height={16} />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="프로젝트 검색"
              className="flex-1 bg-transparent text-sm text-fg placeholder:text-fg-subtle focus:outline-none"
            />
          </div>
        </div>
      )}

      <nav className="mt-3 flex-1 overflow-y-auto px-2">
        {!collapsed && (
          <div className="px-2 pb-1 pt-3 text-xs font-medium uppercase tracking-wider text-fg-subtle">
            프로젝트
          </div>
        )}
        <ul className="space-y-0.5">
          {filtered.length === 0 && !collapsed && (
            <li className="px-2 py-3 text-xs text-fg-subtle">
              아직 프로젝트가 없습니다.
            </li>
          )}
          {filtered.map((p) => {
            const isActive =
              activeProjectId === p.id || location.pathname === `/projects/${p.id}`;
            return (
              <li key={p.id}>
                <div
                  className={cn(
                    "group flex items-center gap-2 rounded-lg px-2 py-1.5 text-sm transition-colors",
                    isActive
                      ? "bg-bg-hover text-fg"
                      : "text-fg-muted hover:bg-bg-hover hover:text-fg",
                  )}
                >
                  <button
                    onClick={() => handleOpenProject(p.id)}
                    className={cn(
                      "flex flex-1 items-center gap-2 truncate text-left",
                      collapsed && "justify-center",
                    )}
                    title={p.name}
                  >
                    <span className="inline-block h-1.5 w-1.5 shrink-0 rounded-full bg-brand/60" />
                    {!collapsed && <span className="truncate">{p.name}</span>}
                  </button>
                  {!collapsed && (
                    <button
                      className="invisible rounded p-1 text-fg-subtle hover:bg-bg hover:text-fg group-hover:visible"
                      onClick={async (e) => {
                        e.stopPropagation();
                        try {
                          const ok = await tauriConfirm(
                            `"${p.name}" 프로젝트를 삭제할까요? 이 작업은 되돌릴 수 없습니다.`,
                            { title: "프로젝트 삭제", kind: "warning" },
                          );
                          if (ok) {
                            await deleteProject(p.id);
                            if (isActive) navigate("/");
                          }
                        } catch (err) {
                          console.error("delete failed", err);
                        }
                      }}
                      title="삭제"
                    >
                      <IconTrash width={14} height={14} />
                    </button>
                  )}
                </div>
              </li>
            );
          })}
        </ul>
      </nav>

      <div className="border-t border-line p-2">
        <button
          className={cn(
            "btn-ghost w-full justify-start",
            collapsed && "justify-center",
          )}
          onClick={() => navigate("/settings")}
        >
          <IconSettings />
          {!collapsed && <span>설정</span>}
        </button>
      </div>
    </aside>
  );
}
