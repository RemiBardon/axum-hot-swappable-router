use std::sync::Arc;

pub async fn listener() -> tokio::net::TcpListener {
    use std::net::Ipv4Addr;
    use tokio::net::TcpListener;

    let addr = (Ipv4Addr::LOCALHOST, 8080);
    TcpListener::bind(addr).await.unwrap()
}

pub async fn get_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum AppStatus {
    Starting,
    Running,
    Unavailable,
}

impl Default for AppStatus {
    fn default() -> Self {
        Self::Starting
    }
}

impl From<&AppStatus> for axum::http::StatusCode {
    fn from(status: &AppStatus) -> Self {
        match status {
            AppStatus::Starting => Self::TOO_EARLY,
            AppStatus::Running => Self::OK,
            AppStatus::Unavailable => Self::SERVICE_UNAVAILABLE,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub(crate) struct AppState<MinimalAppState: Clone + Default> {
    pub base: MinimalAppState,
    pub config: AppConfig,
    pub field1: Arc<String>,
    pub field2: Arc<String>,
    pub field3: Arc<String>,
    pub field4: Arc<String>,
    pub field5: Arc<String>,
    pub field6: Arc<String>,
    pub field7: Arc<String>,
    pub field8: Arc<String>,
    pub field9: Arc<String>,
}

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub(crate) struct AppConfig {
    pub field1: String,
    pub field2: String,
    pub field3: String,
    pub field4: String,
    pub field5: String,
    pub field6: String,
    pub field7: String,
    pub field8: String,
    pub field9: String,
}
