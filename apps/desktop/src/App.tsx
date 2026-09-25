import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface TaskItem {
  id: string;
  source: string;
  destination: string;
  state: string;
  downloaded_bytes: number;
  total_bytes: number | null;
  error: string | null;
}

type Tab = "list" | "add" | "settings";

export default function App() {
  const [server, setServer] = useState("127.0.0.1:39100");
  const [tasks, setTasks] = useState<TaskItem[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("list");
  const [loading, setLoading] = useState(false);
  const [newId, setNewId] = useState("");
  const [newSource, setNewSource] = useState("");
  const [newDest, setNewDest] = useState("");
  const [status, setStatus] = useState("");

  const loadTasks = async () => {
    setLoading(true);
    setStatus("");
    try {
    const result: any[] = await invoke("task_list", { server });
      setTasks(result as TaskItem[]);
    } catch (e) {
      setStatus(`Failed to connect to server: ${e}`);
      setTasks([]);
    }
    setLoading(false);
  };

  useEffect(() => { loadTasks(); }, [server]);

  const handleCreate = async () => {
    setLoading(true);
    setStatus("");
    try {
      await invoke("task_create", { server, id: newId, source: newSource, destination: newDest });
      setNewId(""); setNewSource(""); setNewDest("");
      setStatus("Task created");
      await loadTasks();
    } catch (e) {
      setStatus(`Failed: ${e}`);
    }
    setLoading(false);
  };

  const handleQueue = async (id: string) => {
    try {
      await invoke("task_queue", { server, task_id: id });
      await loadTasks();
    } catch (e) { setStatus(`Failed: ${e}`); }
  };

  const handleStart = async () => {
    try {
      await invoke("task_start", { server });
      await loadTasks();
    } catch (e) { setStatus(`Failed: ${e}`); }
  };

  const handlePause = async (id: string) => {
    try {
      await invoke("task_pause", { server, task_id: id });
      await loadTasks();
    } catch (e) { setStatus(`Failed: ${e}`); }
  };

  const handleResume = async (id: string) => {
    try {
      await invoke("task_resume", { server, task_id: id });
      await loadTasks();
    } catch (e) { setStatus(`Failed: ${e}`); }
  };

  const handleRemove = async (id: string) => {
    try {
      await invoke("task_remove", { server, task_id: id });
      await loadTasks();
    } catch (e) { setStatus(`Failed: ${e}`); }
  };

  const selectedTask = tasks.find((t) => t.id === selected);

  return (
    <div style={{ fontFamily: "system-ui, sans-serif", height: "100vh", display: "flex", flexDirection: "column" }}>
      {/* Header */}
      <div style={{ background: "#1a1a2e", color: "white", padding: "12px 16px", display: "flex", alignItems: "center", gap: "12px" }}>
        <h1 style={{ margin: 0, fontSize: "18px" }}>Nexum</h1>
        <input
          value={server}
          onChange={(e) => setServer(e.target.value)}
          style={{ flex: 1, padding: "6px 10px", borderRadius: "6px", border: "1px solid #444", background: "#16213e", color: "white" }}
        />
        <button onClick={loadTasks} disabled={loading} style={{ padding: "6px 14px", borderRadius: "6px" }}>
          {loading ? "..." : "Refresh"}
        </button>
      </div>

      {/* Tabs */}
      <div style={{ display: "flex", gap: "4px", padding: "8px 16px", borderBottom: "1px solid #ddd" }}>
        {(["list", "add", "settings"] as Tab[]).map((t) => (
          <button key={t} onClick={() => setTab(t)}
            style={{ padding: "6px 16px", borderRadius: "6px 6px 0 0", fontWeight: tab === t ? 600 : 400 }}>
            {t === "list" ? "Tasks" : t === "add" ? "Add" : "Settings"}
          </button>
        ))}
      </div>

      {/* Status */}
      {status && <div style={{ padding: "8px 16px", background: "#fff3cd", fontSize: "13px" }}>{status}</div>}

      {/* Content */}
      <div style={{ flex: 1, overflow: "auto", padding: "16px" }}>
        {tab === "list" && (
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            {tasks.length === 0 && (
              <p style={{ color: "#666" }}>No tasks yet. Add one or connect to a server.</p>
            )}
            {tasks.map((t) => (
              <div key={t.id}
                onClick={() => setSelected(selected === t.id ? null : t.id)}
                style={{ border: selected === t.id ? "2px solid #4361ee" : "1px solid #ddd", borderRadius: "8px", padding: "12px", cursor: "pointer" }}>
                <div style={{ display: "flex", justifyContent: "space-between" }}>
                  <span style={{ fontWeight: 500 }}>{t.id}</span>
                  <span style={{ fontSize: "12px", color: "#999" }}>{t.state}</span>
                </div>
                <div style={{ fontSize: "13px", color: "#666", marginTop: "4px" }}>
                  {t.source} → {t.destination}
                </div>
                <div style={{ fontSize: "12px", color: "#999", marginTop: "4px" }}>
                  {formatBytes(t.downloaded_bytes)} / {formatBytesMaybe(t.total_bytes)}
                </div>
                {t.error && (
                  <div style={{ fontSize: "12px", color: "#b42318", marginTop: "4px" }}>
                    {t.error}
                  </div>
                )}
                <div style={{ display: "flex", gap: "6px", marginTop: "8px" }}>
                  {t.state !== "Queued" && t.state !== "Downloading" && t.state !== "Paused" && (
                    <button onClick={(e) => { e.stopPropagation(); handleQueue(t.id); }}>Queue</button>
                  )}
                  {t.state === "Queued" && (
                    <button onClick={(e) => { e.stopPropagation(); handleStart(); }}>Start</button>
                  )}
                  {t.state === "Downloading" && (
                    <button onClick={(e) => { e.stopPropagation(); handlePause(t.id); }}>Pause</button>
                  )}
                  {(t.state === "Paused" || t.state === "Failed") && (
                    <button onClick={(e) => { e.stopPropagation(); handleResume(t.id); }}>Resume</button>
                  )}
                  <button style={{ color: "red" }} onClick={(e) => { e.stopPropagation(); handleRemove(t.id); }}>Remove</button>
                </div>
              </div>
            ))}
          </div>
        )}

        {tab === "add" && (
          <div style={{ maxWidth: "500px", margin: "0 auto" }}>
            <h3 style={{ marginTop: 0 }}>Add Download Task</h3>
            <label style={{ display: "block", marginBottom: "4px", fontWeight: 500 }}>Task ID</label>
            <input value={newId} onChange={(e) => setNewId(e.target.value)} style={{ width: "100%", padding: "8px", marginBottom: "12px", boxSizing: "border-box" }} placeholder="e.g. my-file" />
            <label style={{ display: "block", marginBottom: "4px", fontWeight: 500 }}>Source URL</label>
            <input value={newSource} onChange={(e) => setNewSource(e.target.value)} style={{ width: "100%", padding: "8px", marginBottom: "12px", boxSizing: "border-box" }} placeholder="https://example.com/file" />
            <label style={{ display: "block", marginBottom: "4px", fontWeight: 500 }}>Destination</label>
            <input value={newDest} onChange={(e) => setNewDest(e.target.value)} style={{ width: "100%", padding: "8px", marginBottom: "16px", boxSizing: "border-box" }} placeholder="/path/to/destination" />
            <button onClick={handleCreate} disabled={loading || !newId || !newSource || !newDest} style={{ padding: "10px 24px" }}>
              {loading ? "Creating..." : "Create Task"}
            </button>
          </div>
        )}

        {tab === "settings" && (
          <div style={{ maxWidth: "400px", margin: "0 auto" }}>
            <h3>Settings</h3>
            <label style={{ display: "block", marginBottom: "4px" }}>Default Server</label>
            <input value={server} onChange={(e) => setServer(e.target.value)} style={{ width: "100%", padding: "8px", boxSizing: "border-box" }} />
            <p style={{ fontSize: "12px", color: "#999" }}>
              Change this to connect to a different Nexum server. The default is 127.0.0.1:39100.
            </p>
          </div>
        )}
      </div>
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
