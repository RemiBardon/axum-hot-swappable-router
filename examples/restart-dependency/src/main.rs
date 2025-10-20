use std::sync::atomic::AtomicU8;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::routing::post;
use axum::Json;
use axum::{routing::get, Router};
use tokio::sync::RwLock;

fn base_router() -> Router<MinimalAppState> {
    println!("Get base_router.");

    Router::new().route("/health", get(health))
}

fn normal_router() -> Router<AppState> {
    println!("Get normal_router.");

    Router::new()
        .route("/reload", post(reload))
        .route("/restart-dependency", post(restart_dependency))
        .route("/users", get(get_users))
}

fn app_misconfigured_router() -> Router<MinimalAppState> {
    println!("Get app_misconfigured_router.");

    Router::new()
        .route("/reload", post(reload))
        .fallback(async move || {
            use serde_json::json;

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": true,
                    "message": "Invalid app config. Check your logs.",
                })),
            )
        })
}

fn server_restarting_router() -> Router {
    println!("Get server_restarting_router.");

    Router::new().fallback(async move || {
        use serde_json::json;

        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": true,
                "message": "Server restarting. Please wait.",
            })),
        )
    })
}

fn server_unavailable_router() -> Router<AppState> {
    println!("Get server_unavailable_router.");

    Router::new()
        .route("/restart-dependency", post(restart_dependency))
        .fallback(async move || {
            use serde_json::json;

            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": true,
                    "message": "Server unavailable. Check your logs.",
                })),
            )
        })
}

use self::app_state::*;
mod app_state {
    use std::sync::Arc;

    use axum_hot_swappable_router::HotSwappableRouter;
    use tokio::sync::{RwLock, RwLockReadGuard};

    use crate::app_config::AppConfig;

    /// The app state for when i.e. the app config is incorrect.
    #[derive(Debug, Clone)]
    pub(crate) struct MinimalAppState {
        pub status: Arc<RwLock<AppStatus>>,
        pub app_router: HotSwappableRouter,
    }

    impl MinimalAppState {
        pub async fn status<'a>(&'a self) -> RwLockReadGuard<'a, AppStatus> {
            self.status.read().await
        }

        pub async fn set_status(&self, new_status: AppStatus) {
            *self.status.write().await = new_status
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct AppState {
        pub base: MinimalAppState,
        #[allow(unused)]
        pub app_config: Arc<AppConfig>,
        pub dependency: DependencyState,
    }

    #[derive(Debug, Clone)]
    pub(crate) enum AppStatus {
        Starting,
        Running,
        Restarting,
        RestartFailed,
        Misconfigured(String),
    }

    /// Simulate a dependency that needs to read a config key
    /// as part of its state.
    #[derive(Debug, Clone)]
    pub(crate) struct DependencyState {
        pub server_hostname: String,
    }

    impl DependencyState {
        pub fn from_config(app_config: &AppConfig) -> Self {
            Self {
                server_hostname: app_config.server.local_hostname.clone(),
            }
        }
    }

    impl axum::extract::FromRef<AppState> for MinimalAppState {
        fn from_ref(app_state: &AppState) -> Self {
            app_state.base.clone()
        }
    }

    impl std::ops::Deref for AppState {
        type Target = MinimalAppState;

        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }
}

use self::app_config::*;
mod app_config {
    use std::net::IpAddr;

    use serde::Deserialize;

    #[derive(Debug)]
    #[derive(Deserialize)]
    pub struct AppConfig {
        pub api: ApiConfig,
        pub server: ServerConfig,
    }

    #[derive(Debug, Clone)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    #[allow(unused)]
    pub struct LogConfig {
        pub level: LogLevel,
        pub format: LogFormat,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
    #[derive(strum::Display, strum::EnumString)]
    #[strum(serialize_all = "snake_case")]
    pub enum LogLevel {
        Trace,
        Debug,
        Info,
        Warn,
        Error,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)]
    #[derive(strum::Display, strum::EnumString)]
    #[strum(serialize_all = "snake_case")]
    pub enum LogFormat {
        Full,
        Compact,
        Json,
        Pretty,
    }

    #[derive(Debug, Clone)]
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ApiConfig {
        pub address: IpAddr,
        pub port: u16,
        #[allow(unused)]
        pub log: LogConfig,
    }

    #[derive(Debug, Clone)]
    #[derive(Deserialize)]
    pub struct ServerConfig {
        #[allow(unused)]
        pub log: LogConfig,
        pub local_hostname: String,
    }
}

use self::routes::*;
mod routes {
    use std::{ops::Deref as _, sync::Arc};

    use axum::{
        extract::{Query, State},
        http::StatusCode,
    };
    use serde::Deserialize;

    use crate::{
        app_misconfigured_router,
        app_state::{AppState, DependencyState},
        normal_router, read_app_config, server_restarting_router, server_unavailable_router,
        AppStatus, MinimalAppState,
    };

    pub async fn health(State(app_state): State<MinimalAppState>) -> StatusCode {
        println!("GET /health");

        match app_state.status().await.deref() {
            AppStatus::Starting => StatusCode::TOO_EARLY,
            AppStatus::Running => StatusCode::OK,
            AppStatus::Restarting | AppStatus::RestartFailed => StatusCode::SERVICE_UNAVAILABLE,
            AppStatus::Misconfigured(reason) => {
                eprintln!("App misconfigured: {reason:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    pub async fn get_users(State(app_state): State<AppState>) -> StatusCode {
        println!("GET /users");

        let ref server_hostname = app_state.dependency.server_hostname;
        let _pretend = format!("Call `GET {server_hostname}/users`.");

        StatusCode::OK
    }

    pub async fn reload(State(state): State<MinimalAppState>) -> StatusCode {
        println!("POST /reload");

        let app_router = state.app_router.clone();
        match read_app_config() {
            Ok(app_config) => {
                state.set_status(AppStatus::Running).await;

                let app_state = AppState {
                    base: state,
                    dependency: DependencyState::from_config(&app_config),
                    app_config: Arc::new(app_config),
                };
                app_router.set(normal_router().with_state(app_state));

                StatusCode::OK
            }
            Err(err) => {
                let new_status = AppStatus::Misconfigured(format!("{err:#}").replace("\n", " "));
                state.set_status(new_status).await;

                app_router.set(app_misconfigured_router().with_state(state));

                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    #[derive(Deserialize)]
    pub struct RestartQuery {
        #[serde(default)]
        failing: bool,
    }

    pub async fn restart_dependency(
        State(state): State<AppState>,
        Query(query): Query<RestartQuery>,
    ) -> StatusCode {
        println!("POST /restart-dependency");

        use tokio::time::Duration;

        let app_router = state.base.app_router.clone();

        state.set_status(AppStatus::Restarting).await;

        let ref server_hostname = state.dependency.server_hostname;

        app_router.set(server_restarting_router());

        println!("Restarting server…");
        let _pretend = format!("Call `PUT {server_hostname}/restart`.");

        if query.failing {
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("Server failed restarting.");

            state.set_status(AppStatus::RestartFailed).await;
            app_router.set(server_unavailable_router().with_state(state));

            StatusCode::SERVICE_UNAVAILABLE
        } else {
            tokio::time::sleep(Duration::from_secs(3)).await;
            println!("Server restarted.");

            state.set_status(AppStatus::Running).await;
            app_router.set(normal_router().with_state(state));

            StatusCode::OK
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    use axum_hot_swappable_router::HotSwappableRouter;
    use tokio::net::TcpListener;

    let hot_swappable_router = HotSwappableRouter::default();
    let app_status = Arc::new(RwLock::new(AppStatus::Starting));

    let base_state = MinimalAppState {
        status: app_status.clone(),
        app_router: hot_swappable_router.clone(),
    };
    let base_router: Router = base_router().with_state(base_state.clone());

    // In production, this would be read from a configuration file,
    // with default values and all. This is just an example.
    let app_config = match read_app_config() {
        Ok(app_config) => app_config,
        Err(err) => {
            return Err(format!("Invalid app config: {err:#}"));
        }
    };

    // Bind to the desired address, and crash if not available.
    let addr = (app_config.api.address, app_config.api.port);
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            return Err(format!("Could not listen to {addr:?}: {err:#}"));
        }
    };

    let app_state = AppState {
        base: base_state.clone(),
        dependency: DependencyState::from_config(&app_config),
        app_config: Arc::new(app_config),
    };
    hot_swappable_router.set(normal_router().with_state(app_state));

    let app = base_router.fallback_service(hot_swappable_router);

    let handle = tokio::spawn(async move {
        println!(
            "Serving API on http://{address}:{port}…",
            address = addr.0,
            port = addr.1
        );
        axum::serve(listener, app).await
    });

    *app_status.write().await = AppStatus::Running;
    println!("Set status to Running.");

    match handle.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => return Err(format!("{err:#}")),
        Err(err) => return Err(format!("{err:#}")),
    }
}

// MARK: - Helpers

static FAKE_CONFIG_RELOAD_COUNTER: AtomicU8 = AtomicU8::new(0);
fn read_app_config() -> Result<AppConfig, toml::de::Error> {
    use std::sync::atomic::Ordering;

    fn valid_app_config() -> toml::map::Map<String, toml::Value> {
        toml::toml! {
            [api]
            address = "127.0.0.1"
            port = 8080
            log = { level = "debug", format = "pretty" }

            [server]
            log = { level = "info", format = "pretty" }
            local_hostname = "server"
        }
    }

    fn invalid_app_config() -> toml::map::Map<String, toml::Value> {
        toml::toml! {
            [api]
            address = "127.0.0.1"
            port = 8080
            log = { level = "debug" }

            [server]
            log = { level = "info" }
            // local_hostname = "server"
        }
    }

    // Alternate between valid and invalid config
    // to pretend admin makes mistakes.
    let reload_count = FAKE_CONFIG_RELOAD_COUNTER.fetch_add(1, Ordering::Relaxed);
    let app_config_toml = if reload_count.is_multiple_of(2) {
        println!("Reading valid app config…");
        valid_app_config()
    } else {
        println!("Reading invalid app config…");
        invalid_app_config()
    };

    app_config_toml.try_into()
}
