import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { CloseIcon, SpinnerIcon } from "./icons";
import type { DiagnosticReport, Finding, Severity } from "./types";

type Props = {
  toolId: string | null;
  report: DiagnosticReport | null;
  loading: boolean;
  onClose: () => void;
};

export function DiagnosticModal(props: Props) {
  const { toolId, report, loading, onClose } = props;
  const [keyResult, setKeyResult] = useState<Finding | null>(null);
  const [keyChecking, setKeyChecking] = useState(false);
  if (!toolId) return null;

  // opt-in：只在用户点按钮时才读 env 并联网校验 key（GET /v1/models，免费）。
  async function verifyKey() {
    setKeyChecking(true);
    try {
      const f = await invoke<Finding>("verify_anthropic_key");
      setKeyResult(f);
    } catch (e) {
      setKeyResult({
        severity: "error",
        message: `校验失败：${String(e)}`,
        suggestion: null,
      });
    } finally {
      setKeyChecking(false);
    }
  }

  const summary = report
    ? {
        ok: report.findings.filter((f) => f.severity === "ok").length,
        warn: report.findings.filter((f) => f.severity === "warn").length,
        err: report.findings.filter((f) => f.severity === "error").length,
      }
    : null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-ink/30 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="card-enter relative w-full max-w-md rounded-2xl border border-line bg-surface p-6 shadow-[0_20px_60px_rgba(31,30,27,0.18)]"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-[16px] font-semibold text-ink">
              诊断报告 · {toolId}
            </h2>
            {summary && (
              <p className="mt-1 text-[12px] text-ink-muted">
                {summary.ok} 项通过 ·{" "}
                <span className={summary.warn > 0 ? "text-warn" : ""}>
                  {summary.warn} 警告
                </span>{" "}
                ·{" "}
                <span className={summary.err > 0 ? "text-danger" : ""}>
                  {summary.err} 错误
                </span>
              </p>
            )}
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md p-1 text-ink-faint transition hover:bg-surface-sunken hover:text-ink"
            aria-label="关闭"
          >
            <CloseIcon className="h-4 w-4" />
          </button>
        </div>

        {toolId === "claude-code" && (
          <div className="mt-4 rounded-lg border border-line bg-surface-sunken/40 p-3">
            <div className="flex items-center justify-between gap-3">
              <p className="text-[12px] text-ink-muted">
                真实校验 API key 有效性（联网 · 免费）
              </p>
              <button
                type="button"
                onClick={verifyKey}
                disabled={keyChecking}
                className="rounded-md border border-line bg-surface px-2.5 py-1 text-[12px] text-ink transition hover:border-ink-muted disabled:opacity-50"
              >
                {keyChecking ? "校验中…" : "验证 Key"}
              </button>
            </div>
            {keyResult && (
              <div
                className={
                  "mt-2 flex items-start gap-2 rounded-md border p-2 " +
                  findingClass(keyResult.severity)
                }
              >
                <SeverityDot severity={keyResult.severity} />
                <div className="min-w-0">
                  <p className="text-[12px] text-ink">{keyResult.message}</p>
                  {keyResult.suggestion && (
                    <p className="mt-0.5 text-[11px] text-ink-muted">
                      💡 {keyResult.suggestion}
                    </p>
                  )}
                </div>
              </div>
            )}
          </div>
        )}

        <div className="mt-5 max-h-[60vh] overflow-auto">
          {loading ? (
            <div className="flex items-center gap-2 py-6 text-[13px] text-ink-muted">
              <SpinnerIcon className="h-3.5 w-3.5 animate-spin" />
              正在检查…
            </div>
          ) : report?.findings.length === 0 ? (
            <p className="py-6 text-center text-[13px] text-ink-faint">
              该工具没有可用的诊断规则。
            </p>
          ) : (
            <ul className="space-y-2">
              {report?.findings.map((f, i) => (
                <li
                  key={i}
                  className={
                    "rounded-lg border p-3 " +
                    findingClass(f.severity)
                  }
                >
                  <div className="flex items-start gap-2">
                    <SeverityDot severity={f.severity} />
                    <div className="min-w-0 flex-1">
                      <p className="text-[13px] text-ink">{f.message}</p>
                      {f.suggestion && (
                        <p className="mt-1 text-[12px] leading-relaxed text-ink-muted">
                          💡 {f.suggestion}
                        </p>
                      )}
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}

function findingClass(s: Severity): string {
  switch (s) {
    case "ok":
      return "border-success-soft/40 bg-success-soft/30";
    case "warn":
      return "border-warn/20 bg-warn/5";
    case "error":
      return "border-danger-soft bg-danger-soft/40";
  }
}

function SeverityDot(props: { severity: Severity }) {
  return (
    <span
      className={
        "mt-1 h-2 w-2 shrink-0 rounded-full " +
        (props.severity === "ok"
          ? "bg-success"
          : props.severity === "warn"
            ? "bg-warn"
            : "bg-danger")
      }
    />
  );
}
