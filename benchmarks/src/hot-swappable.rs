mod shared;

use std::{ops::Deref as _, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, put},
    Router,
};
use axum_hot_swappable_router::HotSwappableRouter;
use tokio::sync::RwLock;

use self::shared::*;

#[derive(Debug, Clone, Default)]
struct MinimalAppState {
    status: Arc<RwLock<AppStatus>>,
    app_router: HotSwappableRouter,
}

fn base_router() -> Router<MinimalAppState> {
    Router::new().route(
        "/health",
        get(|State(state): State<MinimalAppState>| async move {
            StatusCode::from(state.status.read().await.deref())
        }),
    )
}

fn state1_router() -> Router<AppState<MinimalAppState>> {
    Router::new().route("/time", get(get_time)).route(
        "/swap",
        put(async |State(state): State<AppState<MinimalAppState>>| {
            state.base.app_router.set(state2_router());
        }),
    )
}

fn state2_router() -> Router {
    Router::new()
        .route("/bye", get(|| async { "Bye" }))
        .with_state(AppState::<MinimalAppState>::default())
}

#[tokio::main]
async fn main() {
    let hot_swappable_router = HotSwappableRouter::default();

    let base_state = MinimalAppState {
        app_router: hot_swappable_router.clone(),
        ..Default::default()
    };
    let base_router: Router = base_router().with_state(base_state.clone());

    let app_state = AppState {
        base: base_state,
        ..Default::default()
    };
    hot_swappable_router.set(state1_router().with_state(app_state));

    let app: Router = base_router.fallback_service(hot_swappable_router);

    let listener = self::shared::listener().await;
    let handle = tokio::spawn(async { axum::serve(listener, app).await });

    if let Err(err) = handle.await.unwrap() {
        eprintln!("{err:#}");
    }
}
