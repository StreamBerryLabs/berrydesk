use crate::error::Result;
use crate::source::{Protocol, Source};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

pub struct Pipeline {
    source: Source,
    device: String,
    child: Arc<std::sync::Mutex<Option<Child>>>,
    running: Arc<AtomicBool>,
    frames_written: Arc<AtomicU64>,
    error: Arc<std::sync::Mutex<Option<String>>>,
    width: u32,
    height: u32,
    fps: u32,
    stop_flag: Arc<AtomicBool>,
    thread: Arc<std::sync::Mutex<Option<std::thread::JoinHandle<()>>>>,
    snapshot_mode: bool,
    snapshot_thread: Arc<std::sync::Mutex<Option<std::thread::JoinHandle<()>>>>,
    snapshot_temp: Option<std::path::PathBuf>,
}

impl Pipeline {
    pub fn new(source: Source, device: String) -> Self {
        let (w, h) = source.resolution.unwrap_or((1280, 720));
        let fps = source.fps.unwrap_or(30);
        let snapshot_mode = source.protocol == Protocol::HttpSnapshot;
        let snapshot_temp = if snapshot_mode {
            let mut p = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
            p.push("berrydesk");
            let _ = std::fs::create_dir_all(&p);
            p.push("snapshot.jpg");
            Some(p)
        } else {
            None
        };
        Self {
            source,
            device,
            child: Arc::new(std::sync::Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
            frames_written: Arc::new(AtomicU64::new(0)),
            error: Arc::new(std::sync::Mutex::new(None)),
            width: w,
            height: h,
            fps,
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread: Arc::new(std::sync::Mutex::new(None)),
            snapshot_mode,
            snapshot_thread: Arc::new(std::sync::Mutex::new(None)),
            snapshot_temp,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }

    pub fn frames_written(&self) -> u64 {
        self.frames_written.load(Ordering::Relaxed)
    }

    pub fn source_id(&self) -> &str {
        &self.source.id
    }

    pub fn source_name(&self) -> &str {
        &self.source.name
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }
        self.running.store(true, Ordering::Relaxed);
        self.stop_flag.store(false, Ordering::Relaxed);

        // initial snapshot download
        if self.snapshot_mode {
            let _ = download_snapshot(&self.source.url, self.snapshot_temp.as_ref().unwrap());
        }

        let source = self.source.clone();
        let device = self.device.clone();
        let w = self.width;
        let h = self.height;
        let fps = self.fps;
        let running = self.running.clone();
        let child = self.child.clone();
        let error = self.error.clone();
        let stop_flag = self.stop_flag.clone();
        let snapshot_temp = self.snapshot_temp.clone();

        let handle = std::thread::spawn(move || {
            reconnect_loop(source, device, w, h, fps, running.clone(), child.clone(), error.clone(), stop_flag.clone(), snapshot_temp.clone());
        });
        *self.thread.lock().unwrap() = Some(handle);

        if self.snapshot_mode {
            let interval = self.source.snapshot_interval_ms;
            let url = self.source.url.clone();
            let temp = self.snapshot_temp.clone().unwrap();
            let child2 = self.child.clone();
            let running2 = self.running.clone();
            let stop_flag2 = self.stop_flag.clone();
            let h2 = std::thread::spawn(move || {
                while running2.load(Ordering::Relaxed) && !stop_flag2.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(interval as u64));
                    if !running2.load(Ordering::Relaxed) || stop_flag2.load(Ordering::Relaxed) {
                        break;
                    }
                    if download_snapshot(&url, &temp).is_ok() {
                        // kill child to force reload
                        if let Some(mut c) = child2.lock().unwrap().take() {
                            let _ = c.kill();
                            let _ = c.wait();
                        }
                    }
                }
            });
            *self.snapshot_thread.lock().unwrap() = Some(h2);
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(mut c) = self.child.lock().unwrap().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        if let Some(h) = self.thread.lock().unwrap().take() {
            let _ = h.join();
        }
        if let Some(h) = self.snapshot_thread.lock().unwrap().take() {
            let _ = h.join();
        }
    }
}

fn download_snapshot(url: &str, dest: &std::path::Path) -> crate::error::Result<()> {
    let status = Command::new("curl")
        .args(["-sS", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(crate::error::BerryError::Io)?;
    if status.success() {
        Ok(())
    } else {
        Err(crate::error::BerryError::Pipeline(format!("curl failed: {}", status)))
    }
}

#[allow(clippy::too_many_arguments)]
fn reconnect_loop(
    source: Source,
    device: String,
    w: u32,
    h: u32,
    fps: u32,
    running: Arc<AtomicBool>,
    child: Arc<std::sync::Mutex<Option<Child>>>,
    error: Arc<std::sync::Mutex<Option<String>>>,
    stop_flag: Arc<AtomicBool>,
    snapshot_temp: Option<std::path::PathBuf>,
) {
    let mut backoff = Duration::from_secs(1);
    let mut consecutive_failures: u32 = 0;
    let mut use_fallback = false;
    while running.load(Ordering::Relaxed) && !stop_flag.load(Ordering::Relaxed) {
        {
            let mut cg = child.lock().unwrap();
            if cg.is_none() {
                let (eff_source, input_path) = if use_fallback {
                    if let Some(fb) = &source.fallback_image {
                        let mut s = source.clone();
                        s.protocol = Protocol::File;
                        s.url = fb.clone();
                        s.loop_playback = true;
                        (s, std::path::PathBuf::new())
                    } else {
                        (source.clone(), if source.protocol == Protocol::HttpSnapshot {
                            snapshot_temp.clone().unwrap_or_else(|| source.url.clone().into())
                        } else {
                            std::path::PathBuf::new()
                        })
                    }
                } else {
                    (source.clone(), if source.protocol == Protocol::HttpSnapshot {
                        snapshot_temp.clone().unwrap_or_else(|| source.url.clone().into())
                    } else {
                        std::path::PathBuf::new()
                    })
                };
                let args = build_ffmpeg_args(&eff_source, &device, w, h, fps, input_path.as_path());
                info!("ffmpeg args: {}", args.join(" "));
                match Command::new("ffmpeg")
                    .args(&args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(c) => {
                        *cg = Some(c);
                        *error.lock().unwrap() = None;
                        consecutive_failures = 0;
                        backoff = Duration::from_secs(1);
                    }
                    Err(e) => {
                        *error.lock().unwrap() = Some(format!("spawn ffmpeg: {e}"));
                        consecutive_failures += 1;
                        if consecutive_failures >= 3 && source.fallback_image.is_some() {
                            use_fallback = true;
                        }
                        drop(cg);
                        std::thread::sleep(backoff);
                        backoff = (backoff * 2).min(Duration::from_secs(30));
                        continue;
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
        let mut cg = child.lock().unwrap();
        if let Some(c) = cg.as_mut() {
            match c.try_wait() {
                Ok(Some(status)) => {
                    warn!("ffmpeg exited: {}", status);
                    let _ = cg.take();
                    *error.lock().unwrap() = Some(format!("ffmpeg exited: {}", status));
                    consecutive_failures += 1;
                    if consecutive_failures >= 3 && source.fallback_image.is_some() {
                        use_fallback = true;
                    }
                    drop(cg);
                    if running.load(Ordering::Relaxed) && !stop_flag.load(Ordering::Relaxed) {
                        std::thread::sleep(backoff);
                        backoff = (backoff * 2).min(Duration::from_secs(30));
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("wait error: {}", e);
                    let _ = cg.take();
                }
            }
        }
    }
    if let Some(mut c) = child.lock().unwrap().take() {
        let _ = c.kill();
        let _ = c.wait();
    }
}

fn build_ffmpeg_args(src: &Source, device: &str, w: u32, h: u32, fps: u32, snapshot_input: &std::path::Path) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    args.push("-hide_banner".into());
    args.push("-loglevel".into());
    args.push("warning".into());

    // Low-latency input flags (apply before -i)
    match src.protocol {
        Protocol::Rtsp | Protocol::Rtsps => {
            args.push("-fflags".into());
            args.push("nobuffer".into());
            args.push("-flags".into());
            args.push("low_delay".into());
            args.push("-rtsp_transport".into());
            args.push(src.transport.clone().unwrap_or_else(|| "tcp".into()));
            args.push("-probesize".into());
            args.push("500000".into());
            args.push("-analyzeduration".into());
            args.push("1000000".into());
            args.push("-i".into());
            args.push(src.url.clone());
        }
        Protocol::HttpMjpeg => {
            args.push("-fflags".into());
            args.push("nobuffer".into());
            args.push("-i".into());
            args.push(src.url.clone());
        }
        Protocol::TestPattern => {
            args.push("-f".into());
            args.push("lavfi".into());
            args.push("-i".into());
            args.push(format!("testsrc=size={}x{}:rate={}", w, h, fps));
        }
        Protocol::File => {
            if src.loop_playback {
                args.push("-stream_loop".into());
                args.push("-1".into());
            }
            args.push("-i".into());
            args.push(src.url.clone());
        }
        Protocol::HttpSnapshot => {
            args.push("-stream_loop".into());
            args.push("-1".into());
            args.push("-framerate".into());
            args.push(fps.to_string());
            args.push("-i".into());
            args.push(snapshot_input.to_string_lossy().to_string());
        }
        _ => {
            args.push("-fflags".into());
            args.push("nobuffer".into());
            args.push("-i".into());
            args.push(src.url.clone());
        }
    }

    let mut filters: Vec<String> = Vec::new();
    let scale_filter = match src.fit.as_str() {
        "cover" => format!("scale={}:{}:force_original_aspect_ratio=increase,crop={}:{}", w, h, w, h),
        "crop" => format!("crop={}:{}", w, h),
        _ => format!("scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2", w, h, w, h),
    };
    filters.push(scale_filter);
    if src.mirror {
        filters.push("hflip".into());
    }
    match src.rotate {
        90 => filters.push("transpose=1".into()),
        180 => filters.push("transpose=2,transpose=2".into()),
        270 => filters.push("transpose=2".into()),
        _ => {}
    }
    filters.push("format=yuv420p".into());
    args.push("-vf".into());
    args.push(filters.join(","));

    args.push("-r".into());
    args.push(fps.to_string());
    args.push("-f".into());
    args.push("v4l2".into());
    args.push(device.to_string());

    args
}