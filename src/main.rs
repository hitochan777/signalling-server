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
        Box::new(move || {
            println!("checking disconnected peers...");
            let peer_status_repository = peer_status_repository.clone();
            Box::new(async move {
                for user_id in peer_status_repository.fetch_user_ids().await.unwrap() {
                    let statuses = peer_status_repository
                        .fetch_all_by_user_id(user_id)
                        .await
                        .unwrap();

                    for status in statuses {
                        // if status.is_disconnected() {
                        //     println!("{} is disconnected", status.peer_id);
                        // }
                        // leader.disconnect(peer_id)
                    }
                }
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
