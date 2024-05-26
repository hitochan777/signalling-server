use anyhow::Result;
use std::{future::Future, sync::Arc};

use crate::leader_selector;

type Op = Box<dyn Send + Fn() -> Box<dyn Send + Future<Output = ()>>>;

pub struct CronService {
    interval: u64,
    job_fn: Op,
}

impl CronService {
    pub fn new(interval: u64, job_fn: Op) -> Self {
        Self { interval, job_fn }
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for CronService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(self.interval)).await;
            // it is assumed that job_fn executes very quickly, otherwiwse, job_fn is not executed at `interval`
            let future = Box::into_pin((self.job_fn)());
            future.await;
        }
    }
}

type Job = Box<dyn Send + 'static + Fn() -> Box<dyn Send + 'static + Future<Output = ()>>>;

pub fn create_disconnect_check_job(
    peer_status_repository: Arc<Box<dyn leader_selector::PeerStatusRepository>>,
    selector: Arc<leader_selector::LeaderSelector>,
) -> Job {
    Box::new(move || {
        println!("checking disconnected peers...");
        let peer_status_repository = peer_status_repository.clone();
        let selector = selector.clone();
        Box::new(async move {
            for user_id in peer_status_repository.fetch_user_ids().await.unwrap() {
                println!("checking {}", user_id);
                let statuses = peer_status_repository
                    .fetch_all_by_user_id(user_id.clone())
                    .await
                    .unwrap();
                for status in statuses {
                    let now = chrono::Utc::now();
                    if !status.is_valid(now) {
                        println!("{} is disconnected", status.peer_id);
                        let _ = selector
                            .handle_disconnect(user_id.clone(), status.peer_id)
                            .await;
                    }
                }
            }
        })
    })
}
