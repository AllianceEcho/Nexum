import { useState } from "react";

interface TaskItem {
  id: string;
  source: string;
  destination: string;
  state: string;
  downloaded_bytes: number;
  total_bytes: number | null;
}

export default function App() {
  const [tasks, setTasks] = useState<TaskItem[]>([]);
  const [connecting, setConnecting] = useState(false);
  const [server, setServer] = useState("127.0.0.1:39100");

  return (
    <div style={{ fontFamily: "system-ui, sans-serif", padding: "20px" }}>
      <h1 style={{ margin: "0 0 20px" }}>Nexum</h1>

      <div style={{ display: "flex", gap: "8px", marginBottom: "20px" }}>
        <input
          type="text"
          value={server}
          onChange={(e) => setServer(e.target.value)}
          placeholder="Server address"
          style={{ flex: 1, padding: "8px" }}
        />
        <button onClick={() => setConnecting(true)}>Connect</button>
        <button onClick={() => setTasks([])}>Refresh</button>
      </div>

      {tasks.length === 0 ? (
        <p style={{ color: "#666" }}>No tasks. Add a download to get started.</p>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
          {tasks.map((t) => (
            <div key={t.id} style={{
              border: "1px solid #ddd",
              borderRadius: "8px",
              padding: "12px",
            }}>
              <div style={{ fontWeight: 500 }}>{t.id}</div>
              <div style={{ color: "#666", fontSize: "14px" }}>
                {t.source}
              </div>
              <div style={{ fontSize: "12px", color: "#999" }}>
                {t.state} · {formatBytes(t.downloaded_bytes)} / {formatBytesMaybe(t.total_bytes)}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatBytesMaybe(bytes: number | null): string {
  return bytes ? formatBytes(bytes) : "unknown";
}
