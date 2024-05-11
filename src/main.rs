use axum::{
    self,
    extract::{Json, Path, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use shuttle_axum::ShuttleAxum;
use std::sync::Arc;

mod leader_selector;

#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
    let leader_repository: Arc<Box<dyn leader_selector::LeaderRepository>> =
        Arc::new(Box::new(leader_selector::OnMemoryLeaderRepository::new()));
    let peer_status_repository: Arc<Box<dyn leader_selector::PeerStatusRepository>> = Arc::new(
        Box::new(leader_selector::OnMemoryPeerStatusRepository::new()),
    );
    let selector = leader_selector::LeaderSelector::new(peer_status_repository, leader_repository);

    let protected_routes = Router::new()
        .route("/connect/:user_id/:peer_id", post(connect))
        .route("/disconnect/:user_id/:peer_id", post(disconnect))
        .route("/statuses/:user_id", get(get_peer_statuses))
        .with_state(selector);

    let public_routes = Router::new().route("/health", get(|| async { "OK" }));
    let app = protected_routes.merge(public_routes);
    Ok(app.into())
}

async fn connect(
    Path((user_id, peer_id)): Path<(String, String)>,
    State(leader_selector): State<leader_selector::LeaderSelector>,
) -> StatusCode {
    let now = chrono::Utc::now();
    match leader_selector
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
    State(leader_selector): State<leader_selector::LeaderSelector>,
) -> StatusCode {
    match leader_selector.handle_disconnect(user_id, peer_id).await {
        Ok(_) => StatusCode::ACCEPTED,
        Err(err) => {
            println!("Failed to handle disconnection: {:?}", err);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn get_peer_statuses(
    Path(user_id): Path<String>,
    State(leader_selector): State<leader_selector::LeaderSelector>,
) -> Json<Vec<leader_selector::PeerInfo>> {
    match leader_selector.get_statuses_by_user_id(user_id).await {
        Ok(statuses) => Json(statuses),
        Err(err) => {
            println!("Failed to get statuses: {:?}", err);
            Json(vec![])
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
