use std::path::PathBuf;
use anyhow::Result;
use crate::storage::memtable::Observation;
use tokio::io::AsyncWriteExt;

pub struct ChunkStore {
    dir: PathBuf,
}

impl ChunkStore {
    /// Create a new ChunkStore rooted at `data_dir/chunks`.
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let dir = data_dir.join("chunks");
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    /// Write a chunk file for `station_id` with `chunk_name` (for example a date)
    /// Observations are written as newline-delimited JSON (JSONL).
    pub async fn write_chunk(&self, station_id: &str, chunk_name: &str, obs: &[Observation]) -> Result<PathBuf> {
        let fname = format!("{}-{}.ndjson", station_id, chunk_name);
        let path = self.dir.join(fname);
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .await?;

        for o in obs {
            let line = serde_json::to_vec(o)?;
            file.write_all(&line).await?;
            file.write_all(b"\n").await?;
        }
        file.flush().await?;
        Ok(path)
    }

    /// Read all observations for a given `station_id` by scanning chunk files.
    pub async fn read_chunks(&self, station_id: &str) -> Result<Vec<Observation>> {
        let mut out = Vec::new();
        let mut rd = tokio::fs::read_dir(&self.dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            let name = entry.file_name().into_string().unwrap_or_default();
            if !name.starts_with(&format!("{}-", station_id)) {
                continue;
            }
            let data = tokio::fs::read(entry.path()).await?;
            for line in data.split(|b| *b == b'\n') {
                if line.is_empty() { continue; }
                if let Ok(obs) = serde_json::from_slice::<Observation>(line) {
                    out.push(obs);
                }
            }
        }
        Ok(out)
    }

    /// List chunk file paths for a station.
    pub async fn list_chunks(&self, station_id: &str) -> Result<Vec<PathBuf>> {
        let mut res = Vec::new();
        let mut rd = tokio::fs::read_dir(&self.dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            let name = entry.file_name().into_string().unwrap_or_default();
            if name.starts_with(&format!("{}-", station_id)) {
                res.push(entry.path());
            }
        }
        Ok(res)
    }
}
