use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub struct WAL {
    path: PathBuf,
}

impl WAL {
    pub async fn open(path: PathBuf) -> anyhow::Result<Self> {
        // ensure parent exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        // create file if missing
        let _ = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;
        Ok(Self { path })
    }

    pub async fn append(&self, data: &[u8]) -> anyhow::Result<()> {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        file.write_all(data).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;
        Ok(())
    }

    pub async fn replay(&self) -> anyhow::Result<Vec<crate::storage::memtable::Observation>> {
        let content = tokio::fs::read_to_string(&self.path).await.unwrap_or_default();
        let mut out = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(obs) = serde_json::from_str::<crate::storage::memtable::Observation>(line) {
                out.push(obs);
            }
        }
        Ok(out)
    }
}
