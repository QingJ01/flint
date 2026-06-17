import { useState, useRef } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";

type InstallEvent =
  | { type: "Log"; line: string }
  | { type: "Progress"; pct: number }
  | { type: "Done"; ok: boolean; version: string | null; error: string | null };

export default function App() {
  const [node, setNode] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [pct, setPct] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const settled = useRef(false);

  async function detect() {
    try {
      const v = await invoke<string | null>("detect_node");
      setNode(v);
    } catch (e) {
      setLogs((l) => [...l, `✗ 检测失败: ${String(e)}`]);
    }
  }

  async function install() {
    setBusy(true);
    setPct(0);
    setLogs([]);
    settled.current = false;
    const ch = new Channel<InstallEvent>();
    ch.onmessage = (e) => {
      if (e.type === "Log") {
        setLogs((l) => [...l, e.line]);
      } else if (e.type === "Progress") {
        setPct(e.pct);
      } else if (e.type === "Done") {
        if (settled.current) return;
        settled.current = true;
        setBusy(false);
        setPct(100);
        if (e.ok && e.version) {
          setNode(e.version);
          setLogs((l) => [...l, `✓ Node.js ${e.version} 已就绪`]);
        } else {
          setLogs((l) => [...l, `✗ ${e.error ?? "安装未完成"}（可新开终端运行 node -v 复查）`]);
        }
      }
    };
    try {
      await invoke("install_node", { onEvent: ch });
    } catch (err) {
      // Done 事件是权威来源；这里只在尚未 settled 时兜底
      if (!settled.current) {
        settled.current = true;
        setBusy(false);
        setLogs((l) => [...l, `✗ ${String(err)}`]);
      }
    }
  }

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 p-8 font-sans">
      <h1 className="text-2xl font-bold mb-6">Flint</h1>
      <div className="flex items-center gap-4 mb-4">
        <span className={node ? "text-emerald-400" : "text-rose-400"}>
          Node.js: {node ? `✓ ${node}` : "✗ 未安装"}
        </span>
        <button onClick={detect} disabled={busy} className="px-3 py-1 rounded bg-slate-700 disabled:opacity-40">
          检测
        </button>
        <button onClick={install} disabled={busy || !!node} className="px-3 py-1 rounded bg-indigo-600 disabled:opacity-40">
          {busy ? "安装中…" : "安装 Node LTS"}
        </button>
      </div>
      {busy && (
        <div className="w-full bg-slate-800 rounded h-2 mb-4 overflow-hidden">
          <div className="bg-indigo-500 h-2 transition-all" style={{ width: `${pct}%` }} />
        </div>
      )}
      <pre className="bg-black/40 p-4 rounded text-xs h-80 overflow-auto whitespace-pre-wrap">
        {logs.join("\n") || "(暂无日志)"}
      </pre>
    </div>
  );
}
