use axum::Json;
use axum::extract::State;
use serde_json::{Value, json};

use crate::state::AppState;

pub async fn health(State(state): State<AppState>) -> Json<Value> {
    let db = db_ping(&state).await;
    Json(json!({ "status": if db { "ok" } else { "degraded" }, "database": db }))
}

async fn db_ping(state: &AppState) -> bool {
    !state.pool.is_closed() && state.pool.acquire().await.is_ok()
}
