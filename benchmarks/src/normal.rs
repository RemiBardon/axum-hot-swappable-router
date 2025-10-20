mod shared;

use std::{ops::Deref as _, sync::Arc};

use axum::{extract::State, http::StatusCode, routing::get, Router};
use tokio::sync::RwLock;

use self::shared::*;

#[derive(Debug, Clone, Default)]
struct MinimalAppState {
    status: Arc<RwLock<AppStatus>>,
}

fn base_router() -> Router<MinimalAppState> {
    Router::new().route(
        "/health",
        get(|State(state): State<MinimalAppState>| async move {
            StatusCode::from(state.status.read().await.deref())
        }),
    )
}

fn state1_router() -> Router {
    Router::new().route("/time", get(get_time))
}

#[tokio::main]
async fn main() {
    let base_router: Router = base_router().with_state(MinimalAppState::default());

    // Constant router.
    let app: Router = base_router.merge(state1_router());

    let listener = self::shared::listener().await;
    let handle = tokio::spawn(async { axum::serve(listener, app).await });

    if let Err(err) = handle.await.unwrap() {
        eprintln!("{err:#}");
    }
}
