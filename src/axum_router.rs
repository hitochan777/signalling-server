use crate::leader_selector;
use anyhow::anyhow;
use axum::{
    self,
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn create_axum_app(selector: Arc<Box<leader_selector::LeaderSelector>>) -> Router {
    let protected_routes = Router::new()
        .route("/connect/:user_id/:peer_id", post(connect))
        .route("/disconnect/:user_id/:peer_id", post(disconnect))
        .route("/statuses/:user_id", get(get_peer_statuses))
        .route("/leader/:user_id", get(get_leader))
        .with_state(selector);

    let public_routes = Router::new().route("/health", get(|| async { "OK" }));
    return protected_routes.merge(public_routes);
}

async fn connect(
    Path((user_id, peer_id)): Path<(String, String)>,
    State(leader_selector): State<Arc<Box<leader_selector::LeaderSelector>>>,
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
    State(leader_selector): State<Arc<Box<leader_selector::LeaderSelector>>>,
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
    State(leader_selector): State<Arc<Box<leader_selector::LeaderSelector>>>,
) -> Json<Vec<leader_selector::PeerInfo>> {
    match leader_selector.get_statuses_by_user_id(user_id).await {
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
    State(leader_selector): State<Arc<Box<leader_selector::LeaderSelector>>>,
) -> Response {
    match leader_selector
        .get_leader(user_id)
        .await
        .and_then(|opt_string| match opt_string {
            Some(leader_id) => Ok(Some(leader_id)),
            None => Ok(None),
        }) {
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
