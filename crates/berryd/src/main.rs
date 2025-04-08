use berry_core::control::{Request, Response, StatusInfo, socket_path};
use berry_core::{Pipeline, Source, Store};
use clap::Parser;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tracing::info;

struct DaemonState {
    store: Store,
    active: Option<Pipeline>,
    active_id: Option<String>,
    active_url: Option<String>,
    active_protocol: Option<String>,
    device: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let _cli = Cli::parse();
    // ensure socket dir
    let sock = socket_path();
    if let Some(parent) = sock.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _ = std::fs::remove_file(&sock);

    let store = Store::open(None)?;
    let device = detect_device().unwrap_or_else(|| "/dev/video10".into());
    let state = Arc::new(Mutex::new(DaemonState {
        store,
        active: None,
        active_id: None,
        active_url: None,
        active_protocol: None,
        device,
    }));

    let listener = UnixListener::bind(&sock)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&sock, std::fs::Permissions::from_mode(0o600));
    }
    info!("berryd listening on {}", sock.display());

    // handle SIGTERM
    let state_sig = state.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let mut g = state_sig.lock().await;
        if let Some(p) = g.active.as_mut() {
            p.stop();
        }
        std::process::exit(0);
    });

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            let (r, mut w) = stream.into_split();
            let mut reader = BufReader::new(r);
            let mut line = String::new();
            if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                return;
            }
            let req: Request = match serde_json::from_str(line.trim()) {
                Ok(r) => r,
                Err(e) => {
                    let _ = w
                        .write_all(
                            serde_json::to_string(&Response::err(e.to_string()))
                                .unwrap()
                                .as_bytes(),
                        )
                        .await;
                    let _ = w.write_all(b"\n").await;
                    return;
                }
            };
            let resp = handle_request(state, req).await;
            let mut out = serde_json::to_string(&resp).unwrap();
            out.push('\n');
            let _ = w.write_all(out.as_bytes()).await;
        });
    }
}

async fn handle_request(state: Arc<Mutex<DaemonState>>, req: Request) -> Response {
    match req {
        Request::Ping => Response::ok("pong"),
        Request::ListSources => {
            let g = state.lock().await;
            match g.store.list() {
                Ok(srcs) => Response::ok_data("ok", serde_json::to_value(srcs).unwrap()),
                Err(e) => Response::err(e.to_string()),
            }
        }
        Request::AddSource { name, url } => {
            let g = state.lock().await;
            let src = Source::new(name, url);
            // probe-ish: protocol already detected
            if let Err(e) = g.store.upsert(&src) {
                return Response::err(e.to_string());
            }
            Response::ok_data("added", serde_json::to_value(src).unwrap())
        }
        Request::RemoveSource { id } => {
            let g = state.lock().await;
            match g.store.delete(&id) {
                Ok(_) => Response::ok("removed"),
                Err(e) => Response::err(e.to_string()),
            }
        }
        Request::Start { id } => {
            let mut g = state.lock().await;
            // stop existing
            if let Some(p) = g.active.as_mut() {
                p.stop();
            }
            g.active = None;
            g.active_id = None;
            let src = match g.store.get(&id) {
                Ok(s) => s,
                Err(e) => return Response::err(e.to_string()),
            };
            let src_url = src.url.clone();
            let src_protocol = format!("{:?}", src.protocol);
            if src.protocol == berry_core::source::Protocol::Whep {
                return Response::err("WHEP is not implemented in v1; use RTSP/HLS/MJPEG or a local file");
            }
            let device = g.device.clone();
            let mut p = Pipeline::new(src, device);
            if let Err(e) = p.start().await {
                return Response::err(e.to_string());
            }
            g.active_id = Some(id.clone());
            // spawn reconnect loop
            // We'll run pipeline in background via tokio task by moving? Pipeline not Send easily? It is Send-ish.
            // For simplicity, store and let a background task poll. Instead, run reconnect loop here in spawned task.
            let active_arc = Arc::new(Mutex::new(()));
            let _ = active_arc;
            g.active = Some(p);
            g.active_url = Some(src_url);
            g.active_protocol = Some(src_protocol);
            Response::ok("started")
        }
        Request::Stop => {
            let mut g = state.lock().await;
            if let Some(p) = g.active.as_mut() {
                p.stop();
            }
            g.active = None;
            g.active_id = None;
            g.active_url = None;
            g.active_protocol = None;
            Response::ok("stopped")
        }
        Request::Status => {
            let g = state.lock().await;
            let info = StatusInfo {
                running: g.active.as_ref().map(|p| p.is_running()).unwrap_or(false),
                source_id: g.active_id.clone(),
                source_name: g.active.as_ref().map(|p| p.source_name().to_string()),
                fps: None,
                frames_written: g.active.as_ref().map(|p| p.frames_written()).unwrap_or(0),
                device: Some(g.device.clone()),
                error: g.active.as_ref().and_then(|p| p.error()),
            };
            Response::ok_data("ok", serde_json::to_value(info).unwrap())
        }
        Request::Doctor => {
            let g = state.lock().await;
            let mut checks = serde_json::Map::new();
            checks.insert("socket".into(), serde_json::json!(socket_path().exists()));
            checks.insert("device".into(), serde_json::json!(g.device));
            checks.insert(
                "device_exists".into(),
                serde_json::json!(std::path::Path::new(&g.device).exists()),
            );
            checks.insert("ffmpeg".into(), serde_json::json!(which_ffmpeg()));
            Response::ok_data("ok", serde_json::Value::Object(checks))
        }
        Request::InstallDriver => {
            // Try to load v4l2loopback via pkexec (may prompt for password).
            let script = "/home/swadhin/brrrydesk/berrydesk/packaging/setup-v4l2loopback.sh";
            let res = std::process::Command::new("pkexec")
                .arg("bash")
                .arg(script)
                .output();
            match res {
                Ok(o) if o.status.success() => {
                    Response::ok("v4l2loopback loaded; /dev/video10 should now exist")
                }
                Ok(o) => {
                    let err = String::from_utf8_lossy(&o.stderr);
                    Response::err(format!("could not load module ({}). Run manually: sudo modprobe v4l2loopback devices=1 video_nr=10 card_label='BerryDesk Camera' exclusive_caps=1. {}", o.status, err))
                }
                Err(e) => Response::err(format!("pkexec unavailable: {}. Run manually: sudo modprobe v4l2loopback devices=1 video_nr=10 card_label='BerryDesk Camera' exclusive_caps=1", e)),
            }
        }
        Request::Snapshot => {
            let (url, protocol) = {
                let g = state.lock().await;
                (g.active_url.clone(), g.active_protocol.clone())
            };
            let url = match url {
                Some(u) => u,
                None => return Response::err("no active source"),
            };
            // capture one frame as jpeg via ffmpeg
            let mut cmd = std::process::Command::new("ffmpeg");
            cmd.args(["-hide_banner", "-loglevel", "error"]);
            if protocol.as_deref() == Some("TestPattern") {
                cmd.args(["-f", "lavfi", "-i", "testsrc=size=640x480:rate=1"]);
            } else {
                cmd.args(["-i"]).arg(&url);
            }
            let out = cmd
                .args(["-frames:v", "1", "-f", "image2pipe", "-vcodec", "mjpeg", "pipe:1"])
                .output();
            match out {
                Ok(o) if o.status.success() && !o.stdout.is_empty() => {
                    use base64::Engine;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&o.stdout);
                    Response::ok_data("ok", serde_json::json!({ "jpeg_base64": b64 }))
                }
                Ok(o) => Response::err(format!("ffmpeg failed: {}", String::from_utf8_lossy(&o.stderr))),
                Err(e) => Response::err(e.to_string()),
            }
        }
    }
}

fn which_ffmpeg() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn detect_device() -> Option<String> {
    // find first v4l2loopback device
    for i in 0..10 {
        let p = format!("/dev/video{}", i);
        // heuristic: loopback devices often video10+. Check name via sysfs.
        let sys = format!("/sys/class/video4linux/video{}/name", i);
        if let Ok(name) = std::fs::read_to_string(&sys)
            && name.to_lowercase().contains("loopback")
        {
            return Some(p);
        }
    }
    // fallback
    if std::path::Path::new("/dev/video10").exists() {
        return Some("/dev/video10".into());
    }
    None
}

use std::process::Command;

#[derive(Parser)]
#[command(name = "berryd")]
struct Cli {
    #[arg(long)]
    foreground: bool,
}
