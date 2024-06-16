use crate::{leader_selector, pubsub::PubSub};
use anyhow::Result;
use axum::{
    self,
    extract::{Json, Path, State},
    http::StatusCode,
    response::{sse::Event, IntoResponse, Response, Sse},
    routing::{get, post},
    Router,
};
use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_stream::Stream;

struct AppState {
    selector: Arc<leader_selector::LeaderSelector>,
    pubsub: Arc<Mutex<PubSub<SdpEvent>>>,
}

impl AppState {
    fn new_arc(
        selector: Arc<leader_selector::LeaderSelector>,
        pubsub: Arc<Mutex<PubSub<SdpEvent>>>,
    ) -> Arc<Self> {
        Arc::new(Self { selector, pubsub })
    }
}

pub fn create_axum_app(selector: Arc<leader_selector::LeaderSelector>) -> Router {
    let pubsub = Arc::new(Mutex::new(PubSub::new()));
    let protected_routes = Router::new()
        .route("/connect/:user_id/:peer_id", post(connect))
        .route("/disconnect/:user_id/:peer_id", post(disconnect))
        .route("/statuses/:user_id", get(get_peer_statuses))
        .route("/leader/:user_id", get(get_leader))
        .route("/sdp/event", post(handle_sdp_event))
        .route("/sdp/subscribe/:peer_id", get(handle_sse))
        .with_state(AppState::new_arc(selector, pubsub));

    let public_routes = Router::new().route("/health", get(|| async { "OK" }));
    protected_routes.merge(public_routes)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct SdpEvent {
    kind: String,
    from: String,
    to: String,
    sdp: String,
}

async fn handle_sdp_event(
    State(app_state): State<Arc<AppState>>,
    Json(data): Json<SdpEvent>,
) -> StatusCode {
    if app_state
        .pubsub
        .lock()
        .unwrap()
        .publish(&data.to, data.clone())
        .is_ok()
    {
        StatusCode::ACCEPTED
    } else {
        StatusCode::BAD_REQUEST
    }
}

async fn handle_sse(
    Path(peer_id): Path<String>,
    State(app_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut receiver = app_state
        .pubsub
        .lock()
        .unwrap()
        .add_subscriber(&peer_id)
        .unwrap();
    struct Guard {
        app_state: Arc<AppState>,
        peer_id: String,
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            println!("SSE connection {} is gone", self.peer_id);
            self.app_state
                .pubsub
                .lock()
                .unwrap()
                .remove_subscriber(&self.peer_id);
            // TODO: maybe i should call handle_disconnect because it is likely that the SSE disconnection means there is network issue
        }
    }

    let receiver_stream = async_stream::stream! {
        let _guard = Guard {
            app_state: app_state.clone(),
            peer_id: peer_id.clone(),
        };
        loop {
            let event = receiver.recv().await;
            yield Ok(Event::default().data(serde_json::to_string(&event).unwrap()));
        }
        // `_guard` is dropped
    };
    Sse::new(receiver_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn connect(
    Path((user_id, peer_id)): Path<(String, String)>,
    State(app_state): State<Arc<AppState>>,
) -> StatusCode {
    let now = chrono::Utc::now();
    match app_state
        .selector
        .handle_connect(
            user_id,
            leader_selector::PeerInfo {
                peer_id,
                updated_at: now,
            },
        )
        .await
    {
        Ok(_) => StatusCode::ACCEPTED,
        Err(err) => {
            println!("Failed to handle connection: {:?}", err);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn disconnect(
    Path((user_id, peer_id)): Path<(String, String)>,
    State(app_state): State<Arc<AppState>>,
) -> StatusCode {
    match app_state.selector.handle_disconnect(user_id, peer_id).await {
        Ok(_) => StatusCode::ACCEPTED,
        Err(err) => {
            println!("Failed to handle disconnection: {:?}", err);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn get_peer_statuses(
    Path(user_id): Path<String>,
    State(app_state): State<Arc<AppState>>,
) -> Json<Vec<leader_selector::PeerInfo>> {
    match app_state.selector.get_statuses_by_user_id(user_id).await {
        Ok(statuses) => Json(statuses),
        Err(err) => {
            println!("Failed to get statuses: {:?}", err);
            Json(vec![])
        }
    }
}

#[derive(serde::Serialize)]
struct GetLeaderResponse {
    leader_id: Option<String>,
}

async fn get_leader(
    Path(user_id): Path<String>,
    State(app_state): State<Arc<AppState>>,
) -> Response {
    match app_state.selector.get_leader(user_id).await {
        Ok(leader_id) => Json(GetLeaderResponse { leader_id }).into_response(),
        Err(err) => {
            println!("Failed to get leader: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// watcher ---------------> peer_status_service
//    | (disconnect)
//    v
// leader_selector -------> peer_status_service
//    ^
//    | (connect, disconnect, ka)
// API
