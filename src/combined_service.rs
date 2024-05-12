use shuttle_axum::AxumService;

use crate::cron_service::{AsyncFunc, CronService};

pub struct CombinedService<F: AsyncFunc>  {
  axum: AxumService,
  cron: CronService<F>,
}

impl<F: AsyncFunc> CombinedService<F> {
    pub fn new(axum: AxumService, cron: CronService<F>) -> Self {
        Self {
            axum,
            cron,
        }
    }
}

#[shuttle_runtime::async_trait]
impl<F: AsyncFunc> shuttle_runtime::Service for CombinedService<F> {
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

