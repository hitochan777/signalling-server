use axum::{
    self,
    extract::{Json, Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Extension, Router,
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
        .route("/conncet/:user_id/:peer_id", post(connect))
        .route("/disconnect/:user_id/:peer_id", post(disconnect))
        .with_state(selector);

    let public_routes = Router::new().route("/health", get(|| async { "OK" }));
    let app = protected_routes.merge(public_routes);
    Ok(app.into())
}

async fn connect(
    Path(peer_id): Path<String>,
    State(leader_selector): State<leader_selector::LeaderSelector>,
) -> StatusCode {
    match leader_selector
        .handle_connect(
            user_id,
            leader_selector::PeerInfo {
                peer_id,
                updated_at: 0,
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
    Path(peer_id): Path<String>,
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

// watcher ---------------> peer_status_service
//    | (disconnect)
//    v
// leader_selector -------> peer_status_service
//    ^
//    | (connect, disconnect, ka)
// API
