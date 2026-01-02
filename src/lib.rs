use std::sync::Arc;
use tokio::sync::Mutex;
pub mod storage;
pub mod compression;
pub mod api;

pub struct AppState {
    pub memtable: Arc<Mutex<storage::MemTable>>,
    pub wal: Arc<storage::WAL>,
    pub chunk_store: Arc<storage::ChunkStore>,
}

async fn flush_once(state: Arc<AppState>) {
    let chunk_name = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(dur) => dur.as_secs().to_string(),
        Err(_) => "0".to_string(),
    };

    // take ownership of memtable buffer
    let buffer = {
        let mut mt = state.memtable.lock().await;
        if mt.buffer.is_empty() {
            return;
        }
        std::mem::take(&mut mt.buffer)
    };

    for (station_id, obs_vec) in buffer.into_iter() {
        let _ = state.chunk_store.write_chunk(&station_id, &chunk_name, &obs_vec).await;
    }
}

pub async fn run_server() -> anyhow::Result<()> {
    let data_dir = std::path::PathBuf::from("data");
    tokio::fs::create_dir_all(&data_dir).await?;
    let wal = storage::WAL::open(data_dir.join("wal.log")).await?;
    let memtable = storage::MemTable::new();
    let chunk_store = storage::ChunkStore::new(data_dir.clone())?;
    let state = Arc::new(AppState {
        memtable: Arc::new(Mutex::new(memtable)),
        wal: Arc::new(wal),
        chunk_store: Arc::new(chunk_store),
    });

    // bounded flush queue (backpressure) - each item is a vector of (station_id, observations)
    let (flush_tx, mut flush_rx) = tokio::sync::mpsc::channel::<Vec<(String, Vec<storage::memtable::Observation>)>>(2);

    // broadcast channel for shutdown signaling
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // flush worker: consumes queued buffers and writes them sequentially
    {
        let cs = state.chunk_store.clone();
        let mut rx = flush_rx;
        let mut shutdown_sub = shutdown_tx.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_sub.recv() => {
                        // drain remaining items then exit
                        while let Ok(buf) = rx.try_recv() {
                            for (station_id, obs_vec) in buf {
                                let _ = cs.write_chunk(&station_id, "shutdown", &obs_vec).await;
                            }
                        }
                        break;
                    }
                    Some(buf) = rx.recv() => {
                        for (station_id, obs_vec) in buf {
                            let ts = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            let _ = cs.write_chunk(&station_id, &format!("flush-{}", ts), &obs_vec).await;
                        }
                    }
                }
            }
        });
    }

    // periodic scheduler: extract memtable and enqueue for background flush
    {
        let s = state.clone();
        let tx = flush_tx.clone();
        let shutdown_sub = shutdown_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                // take buffer
                let buffer = {
                    let mut mt = s.memtable.lock().await;
                    if mt.buffer.is_empty() { continue; }
                    std::mem::take(&mut mt.buffer)
                };

                // convert into sendable vector
                let mut to_send = Vec::with_capacity(buffer.len());
                for (k, v) in buffer.into_iter() {
                    to_send.push((k, v));
                }

                // try send without blocking; if full, wait up to 2s then give up and reinsert
                match tx.try_send(to_send) {
                    Ok(_) => {}
                    Err(tokio::sync::mpsc::error::TrySendError::Full(buf)) => {
                        let send_fut = tx.send(buf);
                        match tokio::time::timeout(std::time::Duration::from_secs(2), send_fut).await {
                            Ok(Ok(_)) => {}
                            _ => {
                                // backpressure: reinsert observations into memtable to avoid data loss
                                let mut mt = s.memtable.lock().await;
                                for (k, v) in buf {
                                    mt.buffer.entry(k).or_default().extend(v);
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
                // loop continues until shutdown signal
            }
        });
    }

    // run HTTP server in background; it will be shut down via broadcast signal
    let http_state = state.clone();
    let http_shutdown = shutdown_tx.clone();
    tokio::spawn(async move {
        api::http::run(http_state, http_shutdown).await;
    });

    // wait for CTRL-C then signal shutdown
    tokio::signal::ctrl_c().await?;
    let _ = shutdown_tx.send(());
    Ok(())
}
