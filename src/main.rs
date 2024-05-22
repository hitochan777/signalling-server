use axum_router::create_axum_app;
use combined_service::CombinedService;
use cron_service::CronService;
use shuttle_axum::AxumService;
use std::{future::Future, sync::Arc};

mod axum_router;
mod combined_service;
mod cron_service;
mod leader_selector;

#[shuttle_runtime::main]
async fn main() -> Result<CombinedService, shuttle_runtime::Error> {
    let leader_repository: Arc<Box<dyn leader_selector::LeaderRepository>> =
        Arc::new(Box::new(leader_selector::OnMemoryLeaderRepository::new()));
    let peer_status_repository: Arc<Box<dyn leader_selector::PeerStatusRepository>> = Arc::new(
        Box::new(leader_selector::OnMemoryPeerStatusRepository::new()),
    );
    let cloned_peer_status_repository = peer_status_repository.clone();
    let job: Box<dyn Send + 'static + Fn() -> Box<dyn Send + 'static + Future<Output = ()>>> =
        Box::new(|| {
            Box::new(async move {
                peer_status_repository
                    .fetch_one(String::from("peer1"), String::from("peer2"))
                    .await;
                // for user_id in cloned_peer_status_repository.fetch_user_ids().await.unwrap() {
                //     peer_status_repository.fetch_all_by_user_id(user_id).await;
                // }
            })
        });
    Ok(CombinedService::new(
        AxumService::from(create_axum_app(
            leader_repository,
            cloned_peer_status_repository,
        )),
        CronService::new(10, job),
    ))
}
