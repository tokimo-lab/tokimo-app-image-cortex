import { ChevronLeft, ChevronRight, FolderOpen, Image, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

interface FsEntry {
  name: string;
  path: string;
  isDirectory: boolean;
}

interface StorageSource {
  id: string;
  name: string;
  type: string;
}

interface Props {
  onConfirm: (sourceId: string, sourceName: string, filePath: string) => void;
  onCancel: () => void;
  listSources: () => Promise<StorageSource[]>;
  initialSourceId?: string;
  initialPath?: string;
}

const IMAGE_EXTS = new Set(["jpg", "jpeg", "png", "webp", "gif", "bmp", "tiff", "heic", "heif", "avif", "raw", "cr2", "cr3", "nef", "arw", "dng"]);

function isImageFile(name: string): boolean {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return IMAGE_EXTS.has(ext);
}

export function VfsFilePicker({ onConfirm, onCancel, listSources, initialSourceId, initialPath }: Props) {
  const [sources, setSources] = useState<StorageSource[]>([]);
  const [sourceId, setSourceId] = useState(initialSourceId ?? "");
  const [sourceName, setSourceName] = useState("");
  const [history, setHistory] = useState<string[]>([initialPath ?? "/"]);
  const [historyIdx, setHistoryIdx] = useState(0);
  const [entries, setEntries] = useState<FsEntry[]>([]);
  const [parentPath, setParentPath] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const historyIdxRef = useRef(0);
  historyIdxRef.current = historyIdx;

  const currentPath = history[historyIdx] ?? "/";

  useEffect(() => {
    listSources().then(setSources).catch(() => {});
  }, [listSources]);

  useEffect(() => {
    if (initialSourceId && sources.length > 0) {
      const src = sources.find((s) => s.id === initialSourceId);
      if (src) {
        setSourceId(src.id);
        setSourceName(src.name);
      }
    }
  }, [sources, initialSourceId]);

  const browse = useCallback(async (path: string) => {
    if (!sourceId) return;
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`/api/vfs/${sourceId}/browse?path=${encodeURIComponent(path)}`);
      if (!res.ok) throw new Error(await res.text());
      const data = await res.json();
      setEntries(data.entries ?? []);
      setParentPath(data.parentPath ?? null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [sourceId]);

  useEffect(() => {
    browse(currentPath);
  }, [browse, currentPath]);

  const navigate = useCallback((path: string) => {
    setSelectedFile(null);
    const idx = historyIdxRef.current;
    setHistory((prev) => [...prev.slice(0, idx + 1), path]);
    setHistoryIdx(idx + 1);
  }, []);

  const goBack = useCallback(() => {
    if (historyIdxRef.current > 0) {
      setSelectedFile(null);
      setHistoryIdx((prev) => prev - 1);
    }
  }, []);

  const goForward = useCallback(() => {
    if (historyIdxRef.current < history.length - 1) {
      setSelectedFile(null);
      setHistoryIdx((prev) => prev + 1);
    }
  }, [history.length]);

  const handleSourceChange = (id: string) => {
    const src = sources.find((s) => s.id === id);
    setSourceId(id);
    setSourceName(src?.name ?? "");
    setSelectedFile(null);
    setHistory(["/"]);
    setHistoryIdx(0);
  };

  const handleEntryClick = (entry: FsEntry) => {
    if (entry.isDirectory) {
      navigate(entry.path);
    } else {
      setSelectedFile(entry.path);
    }
  };

  const handleConfirm = () => {
    if (sourceId && selectedFile) {
      onConfirm(sourceId, sourceName, selectedFile);
    }
  };

  return (
    <div className="flex flex-col h-full text-[var(--color-fg-primary)]">
      {/* Header: source selector + navigation */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-black/10 dark:border-white/10">
        <select
          value={sourceId}
          onChange={(e) => handleSourceChange(e.target.value)}
          className="rounded border border-black/10 dark:border-white/10 bg-transparent px-2 py-1 text-xs"
        >
          <option value="">选择存储源…</option>
          {sources.map((s) => (
            <option key={s.id} value={s.id}>{s.name} ({s.type})</option>
          ))}
        </select>
        <div className="flex items-center gap-1 ml-2">
          <button
            type="button"
            onClick={goBack}
            disabled={historyIdx <= 0}
            className="cursor-pointer rounded p-1 hover:bg-black/[0.05] dark:hover:bg-white/[0.05] disabled:opacity-30"
          >
            <ChevronLeft size={14} />
          </button>
          <button
            type="button"
            onClick={goForward}
            disabled={historyIdx >= history.length - 1}
            className="cursor-pointer rounded p-1 hover:bg-black/[0.05] dark:hover:bg-white/[0.05] disabled:opacity-30"
          >
            <ChevronRight size={14} />
          </button>
          <button
            type="button"
            onClick={() => browse(currentPath)}
            disabled={loading}
            className="cursor-pointer rounded p-1 hover:bg-black/[0.05] dark:hover:bg-white/[0.05]"
          >
            <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
          </button>
        </div>
        <span className="ml-2 text-xs opacity-50 truncate flex-1">{currentPath}</span>
      </div>

      {/* File list */}
      <div className="flex-1 overflow-y-auto">
        {!sourceId ? (
          <div className="flex items-center justify-center h-full text-sm opacity-40">请先选择存储源</div>
        ) : loading && entries.length === 0 ? (
          <div className="flex items-center justify-center h-full text-sm opacity-40">加载中…</div>
        ) : entries.length === 0 ? (
          <div className="flex items-center justify-center h-full text-sm opacity-40">空目录</div>
        ) : (
          <>
            {parentPath && (
              <button
                type="button"
                onClick={() => navigate(parentPath)}
                className="flex items-center gap-2 px-3 py-1.5 w-full cursor-pointer text-sm hover:bg-black/[0.05] dark:hover:bg-white/[0.05] bg-transparent border-0 text-inherit"
              >
                <FolderOpen size={14} className="opacity-50" />
                <span className="opacity-60">..</span>
              </button>
            )}
            {entries.map((entry) => {
              const isDir = entry.isDirectory;
              const isImg = !isDir && isImageFile(entry.name);
              const isSelected = selectedFile === entry.path;
              return (
                <button
                  key={entry.path}
                  type="button"
                  onClick={() => handleEntryClick(entry)}
                  className={`flex items-center gap-2 px-3 py-1.5 w-full cursor-pointer text-sm text-left bg-transparent border-0 text-inherit ${
                    isSelected
                      ? "bg-[var(--color-accent-subtle)] text-[var(--color-accent)]"
                      : "hover:bg-black/[0.05] dark:hover:bg-white/[0.05]"
                  }`}
                  onDoubleClick={() => isDir && navigate(entry.path)}
                >
                  {isDir ? (
                    <FolderOpen size={14} className="text-amber-500 shrink-0" />
                  ) : isImg ? (
                    <Image size={14} className="text-green-500 shrink-0" />
                  ) : (
                    <span className="w-3.5 shrink-0" />
                  )}
                  <span className={`truncate ${isDir ? "" : isImg ? "" : "opacity-50"}`}>
                    {entry.name}
                  </span>
                </button>
              );
            })}
          </>
        )}
        {error && (
          <div className="mx-3 mt-2 rounded bg-red-500/10 px-2 py-1 text-xs text-red-500">{error}</div>
        )}
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between px-3 py-2 border-t border-black/10 dark:border-white/10">
        <span className="text-xs opacity-50 truncate">
          {selectedFile ? selectedFile.split("/").pop() : "未选择文件"}
        </span>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="cursor-pointer rounded px-3 py-1.5 text-xs hover:bg-black/[0.05] dark:hover:bg-white/[0.05]"
          >
            取消
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            disabled={!selectedFile}
            className="cursor-pointer rounded bg-[var(--color-accent)] px-3 py-1.5 text-xs text-white disabled:opacity-50"
          >
            选择文件
          </button>
        </div>
      </div>
    </div>
  );
}
