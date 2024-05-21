use std::{future::Future, pin::Pin};

use shuttle_axum::AxumService;

use crate::cron_service::CronService;

pub struct CombinedService {
    axum: AxumService,
    cron: CronService,
}

impl CombinedService {
    pub fn new(axum: AxumService, cron: CronService) -> Self {
        Self { axum, cron }
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for CombinedService {
    async fn bind(self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let http = async {
            let _ = self.axum.bind(addr).await;
        };
        let cron = async {
            let _ = self.cron.bind(addr).await;
        };
        tokio::select! {
            _ = http => {}
            _ = cron => {}
        };
        Ok(())
    }
}
