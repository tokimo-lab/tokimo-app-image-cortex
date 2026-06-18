import type { JobStatusResponse } from "../api/client";

interface Props {
  jobId: string;
  result: JobStatusResponse | null;
  t: (key: string) => string;
}

export function JobProgress({ jobId, result, t }: Props) {
  if (!result) {
    return (
      <div className="rounded border border-black/10 dark:border-white/10 p-3">
        <div className="flex items-center gap-2">
          <span className="inline-block h-2 w-2 rounded-full bg-blue-500 animate-pulse" />
          <span className="text-sm opacity-50">
            {t("jobSubmitted")}: {jobId.slice(0, 8)}…
          </span>
        </div>
      </div>
    );
  }

  const statusColor =
    result.status === "completed"
      ? "bg-green-500"
      : result.status === "failed"
        ? "bg-red-500"
        : "bg-blue-500 animate-pulse";

  return (
    <div className="rounded border border-black/10 dark:border-white/10 p-3">
      <div className="flex items-center gap-2 mb-2">
        <span className={`inline-block h-2 w-2 rounded-full ${statusColor}`} />
        <span className="text-sm font-medium">
          {t("analysisStatus")}: {result.status}
        </span>
        {result.status === "completed" && (
          <span className="text-xs opacity-50">#{result.id.slice(0, 8)}</span>
        )}
      </div>
      {result.progress > 0 && result.status !== "completed" && (
        <div className="w-full bg-black/10 dark:bg-white/10 rounded-full h-1.5 mb-2">
          <div
            className="bg-[var(--color-accent)] h-1.5 rounded-full transition-all"
            style={{ width: `${result.progress}%` }}
          />
        </div>
      )}
      {result.error && (
        <div className="text-sm text-red-500 mt-1">{result.error}</div>
      )}
    </div>
  );
}
