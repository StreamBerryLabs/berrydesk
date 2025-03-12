use crate::error::{BerryError, Result};
use crate::source::{Protocol, Source};
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: Option<PathBuf>) -> Result<Self> {
        let path = path.unwrap_or_else(|| {
            let mut p = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            p.push("berrydesk");
            let _ = std::fs::create_dir_all(&p);
            p.push("berrydesk.db");
            p
        });
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sources (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                protocol TEXT NOT NULL,
                transport TEXT,
                resolution TEXT,
                fps INTEGER,
                fit TEXT NOT NULL,
                mirror INTEGER NOT NULL,
                rotate INTEGER NOT NULL,
                fallback_image TEXT,
                loop_playback INTEGER NOT NULL,
                snapshot_interval_ms INTEGER NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn list(&self) -> Result<Vec<Source>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,name,url,protocol,transport,resolution,fps,fit,mirror,rotate,fallback_image,loop_playback,snapshot_interval_ms FROM sources",
        )?;
        let rows = stmt.query_map([], |r| {
            let proto: String = r.get(3)?;
            let res: Option<String> = r.get(5)?;
            let resolution = res.and_then(|s| {
                let parts: Vec<&str> = s.split('x').collect();
                if parts.len() == 2 {
                    Some((parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0)))
                } else {
                    None
                }
            });
            Ok(Source {
                id: r.get(0)?,
                name: r.get(1)?,
                url: r.get(2)?,
                protocol: parse_proto(&proto),
                username: None,
                transport: r.get(4)?,
                resolution,
                fps: r.get(6)?,
                fit: r.get(7)?,
                mirror: r.get::<_, i64>(8)? != 0,
                rotate: r.get(9)?,
                fallback_image: r.get(10)?,
                loop_playback: r.get::<_, i64>(11)? != 0,
                snapshot_interval_ms: r.get::<_, i64>(12)? as u32,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert(&self, s: &Source) -> Result<()> {
        let res = s.resolution.map(|(w, h)| format!("{}x{}", w, h));
        self.conn.execute(
            "INSERT INTO sources (id,name,url,protocol,transport,resolution,fps,fit,mirror,rotate,fallback_image,loop_playback,snapshot_interval_ms)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
             ON CONFLICT(id) DO UPDATE SET name=?2,url=?3,protocol=?4,transport=?5,resolution=?6,fps=?7,fit=?8,mirror=?9,rotate=?10,fallback_image=?11,loop_playback=?12,snapshot_interval_ms=?13",
            params![
                s.id, s.name, s.url, format!("{:?}", s.protocol), s.transport, res, s.fps,
                s.fit, s.mirror as i64, s.rotate, s.fallback_image, s.loop_playback as i64,
                s.snapshot_interval_ms as i64
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM sources WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Source> {
        self.list()?
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| BerryError::NotFound(id.into()))
    }
}

fn parse_proto(s: &str) -> Protocol {
    match s {
        "Rtsp" => Protocol::Rtsp,
        "Rtsps" => Protocol::Rtsps,
        "HttpMjpeg" => Protocol::HttpMjpeg,
        "HttpSnapshot" => Protocol::HttpSnapshot,
        "Hls" => Protocol::Hls,
        "Rtmp" => Protocol::Rtmp,
        "Srt" => Protocol::Srt,
        "Whep" => Protocol::Whep,
        "File" => Protocol::File,
        "TestPattern" => Protocol::TestPattern,
        _ => Protocol::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let _ = std::fs::remove_file("/tmp/scratchpad/test.db");
        let store = Store::open(Some(std::path::PathBuf::from("/tmp/scratchpad/test.db"))).unwrap();
        let src = crate::source::Source::new("demo", "rtsp://x");
        store.upsert(&src).unwrap();
        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "demo");
        store.delete(&src.id).unwrap();
        assert!(store.list().unwrap().is_empty());
    }
}
