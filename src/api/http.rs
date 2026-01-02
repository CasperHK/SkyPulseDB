use axum::{routing::post, Router, Json, extract::Extension};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast::Sender as BroadcastSender;

#[derive(Deserialize)]
pub struct WriteRequest {
    pub station_id: String,
    pub time: String,
    pub temp: Option<f64>,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
    pub wind_speed: Option<f64>,
    pub wind_dir: Option<u16>,
}

pub async fn run(state: Arc<crate::AppState>, shutdown: BroadcastSender<()>) {
    let app = Router::new()
        .route("/api/v1/write", post(write_handler))
        .layer(Extension(state));
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening on http://{}", addr);
    let server = axum::Server::bind(&addr).serve(app.into_make_service());
    let mut shutdown_sub = shutdown.subscribe();
    let graceful = server.with_graceful_shutdown(async move {
        let _ = shutdown_sub.recv().await;
    });
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

async fn write_handler(
    Extension(state): Extension<Arc<crate::AppState>>,
    Json(payload): Json<WriteRequest>,
) -> Json<serde_json::Value> {
    // serialize payload to JSON line for WAL
    let obs = crate::storage::memtable::Observation {
        station_id: payload.station_id.clone(),
        time: payload.time.clone(),
        temp: payload.temp,
        humidity: payload.humidity,
        pressure: payload.pressure,
        wind_speed: payload.wind_speed,
        wind_dir: payload.wind_dir,
    };

    if let Ok(line) = serde_json::to_vec(&obs) {
        let _ = state.wal.append(&line).await;
    }

    // insert into MemTable
    {
        let mut mt = state.memtable.lock().await;
        mt.insert(obs);
    }

    Json(serde_json::json!({"status": "ok"}))
}
