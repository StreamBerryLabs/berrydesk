use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Rtsp,
    Rtsps,
    HttpMjpeg,
    HttpSnapshot,
    Hls,
    Rtmp,
    Srt,
    Whep,
    File,
    TestPattern,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detect_protocols() {
        assert_eq!(Protocol::detect("rtsp://x/stream"), Protocol::Rtsp);
        assert_eq!(Protocol::detect("rtsps://x"), Protocol::Rtsps);
        assert_eq!(Protocol::detect("http://cam/video.mjpg"), Protocol::HttpMjpeg);
        assert_eq!(Protocol::detect("http://cam/snapshot.jpg"), Protocol::HttpSnapshot);
        assert_eq!(Protocol::detect("https://h/live.m3u8"), Protocol::Hls);
        assert_eq!(Protocol::detect("test"), Protocol::TestPattern);
        assert_eq!(Protocol::detect("video.mp4"), Protocol::File);
    }
}

impl Protocol {
    pub fn detect(url: &str) -> Self {
        let u = url.trim().to_lowercase();
        if u == "test" || u == "testpattern" || u == "pattern" {
            return Protocol::TestPattern;
        }
        if u.starts_with("rtsps://") {
            return Protocol::Rtsps;
        }
        if u.starts_with("rtsp://") {
            return Protocol::Rtsp;
        }
        if u.starts_with("rtmp://") || u.starts_with("rtmps://") {
            return Protocol::Rtmp;
        }
        if u.starts_with("srt://") {
            return Protocol::Srt;
        }
        if u.starts_with("file://") {
            return Protocol::File;
        }
        if !u.starts_with("http")
            && (u.ends_with(".mp4")
                || u.ends_with(".mkv")
                || u.ends_with(".mov")
                || u.ends_with(".avi"))
        {
            return Protocol::File;
        }
        if u.contains(".m3u8") {
            return Protocol::Hls;
        }
        if u.contains("whep") {
            return Protocol::Whep;
        }
        if u.ends_with(".mjpg")
            || u.ends_with(".mjpeg")
            || u.contains("mjpg")
            || u.contains("mjpeg")
        {
            return Protocol::HttpMjpeg;
        }
        if u.starts_with("http://") || u.starts_with("https://") {
            // snapshot heuristic: common snapshot paths
            if u.contains("snapshot")
                || u.contains("shot.jpg")
                || u.ends_with(".jpg")
                || u.ends_with(".jpeg")
                || u.ends_with(".png")
            {
                return Protocol::HttpSnapshot;
            }
            return Protocol::HttpMjpeg;
        }
        Protocol::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub url: String,
    pub protocol: Protocol,
    pub username: Option<String>,
    pub transport: Option<String>, // tcp/udp for rtsp
    pub resolution: Option<(u32, u32)>,
    pub fps: Option<u32>,
    pub fit: String, // fit/cover/crop
    pub mirror: bool,
    pub rotate: i32, // 0/90/180/270
    pub fallback_image: Option<String>,
    pub loop_playback: bool,
    pub snapshot_interval_ms: u32,
}

impl Source {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        let url = url.into();
        let protocol = Protocol::detect(&url);
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            url,
            protocol,
            username: None,
            transport: None,
            resolution: None,
            fps: None,
            fit: "fit".into(),
            mirror: false,
            rotate: 0,
            fallback_image: None,
            loop_playback: true,
            snapshot_interval_ms: 500,
        }
    }
}
