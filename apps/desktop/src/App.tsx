import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Source {
  id: string;
  name: string;
  url: string;
  protocol: string;
}

interface Status {
  running: boolean;
  source_id?: string;
  source_name?: string;
  device?: string;
  error?: string;
  frames_written?: number;
}

export default function App() {
  const [sources, setSources] = useState<Source[]>([]);
  const [status, setStatus] = useState<Status>({ running: false });
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [msg, setMsg] = useState("");
  const [doctor, setDoctor] = useState<any>(null);
  const [preview, setPreview] = useState<string>("");

  async function refresh() {
    try {
      const s = await invoke<any[]>("list_sources");
      setSources(s as Source[]);
      const st = await invoke<Status>("status");
      setStatus(st);
      if (st.running) {
        try {
          const d = await invoke<any>("snapshot");
          if (d?.jpeg_base64) setPreview(`data:image/jpeg;base64,${d.jpeg_base64}`);
        } catch {}
      } else {
        setPreview("");
      }
    } catch (e) {
      setMsg(String(e));
    }
  }

  useEffect(() => {
    refresh();
    const t = setInterval(refresh, 1500);
    return () => clearInterval(t);
  }, []);

  async function add() {
    if (!url.trim()) return;
    try {
      await invoke("add_source", { name: name || url, url });
      setName("");
      setUrl("");
      await refresh();
    } catch (e) {
      setMsg(String(e));
    }
  }

  async function start(id: string) {
    try {
      await invoke("start_source", { id });
      await refresh();
    } catch (e) {
      setMsg(String(e));
    }
  }

  async function stop() {
    try {
      await invoke("stop_source");
      await refresh();
    } catch (e) {
      setMsg(String(e));
    }
  }

  async function remove(id: string) {
    try {
      await invoke("remove_source", { id });
      await refresh();
    } catch (e) {
      setMsg(String(e));
    }
  }

  async function runDoctor() {
    try {
      const d = await invoke<any>("doctor");
      setDoctor(d);
    } catch (e) {
      setMsg(String(e));
    }
  }

  return (
    <main className="container">
      <h1>BerryDesk</h1>
      <p className="subtitle">Turn any video feed URL into a system virtual camera: "BerryDesk Camera"</p>

      <section className="card">
        <h2>Add source</h2>
        <div className="row">
          <input placeholder="Name" value={name} onChange={(e) => setName(e.target.value)} />
          <input placeholder="Feed URL or file path (rtsp://, http://, .mp4, 'test')" value={url} onChange={(e) => setUrl(e.target.value)} style={{ flex: 2 }} />
          <button onClick={add}>Add</button>
        </div>
      </section>

      <section className="card">
        <h2>Status</h2>
        <p>
          {status.running
            ? `Running: ${status.source_name ?? status.source_id} on ${status.device ?? "?"}`
            : "Stopped"}
          {status.error && <span className="err"> — {status.error}</span>}
        </p>
        <button onClick={stop} disabled={!status.running}>Stop</button>
        <button onClick={runDoctor}>Doctor</button>
        {preview && <div style={{ marginTop: 12 }}><img src={preview} alt="preview" style={{ maxWidth: "100%", borderRadius: 8 }} /></div>}
        {doctor && <pre className="pre">{JSON.stringify(doctor, null, 2)}</pre>}
      </section>

      <section className="card">
        <h2>Saved sources</h2>
        {sources.length === 0 && <p>No sources yet.</p>}
        <ul className="list">
          {sources.map((s) => (
            <li key={s.id}>
              <div>
                <strong>{s.name}</strong>
                <div className="muted">{s.protocol} — {s.url}</div>
              </div>
              <div className="actions">
                <button onClick={() => start(s.id)}>Start</button>
                <button onClick={() => remove(s.id)}>Remove</button>
              </div>
            </li>
          ))}
        </ul>
      </section>

      {msg && <p className="err">{msg}</p>}
    </main>
  );
}
