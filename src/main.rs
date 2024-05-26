use axum_router::create_axum_app;
use combined_service::CombinedService;
use cron_service::{create_disconnect_check_job, CronService};
use shuttle_axum::AxumService;
use std::sync::Arc;

mod axum_router;
mod combined_service;
mod cron_service;
mod leader_selector;
mod pubsub;

#[shuttle_runtime::main]
async fn main() -> Result<CombinedService, shuttle_runtime::Error> {
    let leader_repository: Arc<Box<dyn leader_selector::LeaderRepository>> =
        Arc::new(Box::new(leader_selector::OnMemoryLeaderRepository::new()));
    let peer_status_repository: Arc<Box<dyn leader_selector::PeerStatusRepository>> = Arc::new(
        Box::new(leader_selector::OnMemoryPeerStatusRepository::new()),
    );
    let selector = Arc::new(leader_selector::LeaderSelector::new(
        peer_status_repository.clone(),
        leader_repository.clone(),
    ));

    Ok(CombinedService::new(
        AxumService::from(create_axum_app(selector.clone())),
        CronService::new(
            10,
            create_disconnect_check_job(peer_status_repository, selector.clone()),
        ),
    ))
}
