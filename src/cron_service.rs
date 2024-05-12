use std::{future::Future, pin::Pin};

type FuncResult = Pin<Box<dyn Future<Output=Result<(), String>>>>;
pub trait AsyncFunc: Fn() -> FuncResult + Send + Sync + 'static {} 

pub struct CronService<F: AsyncFunc> {
  interval: u64,
  job_fn: F, 
}

impl<F: AsyncFunc> CronService<F> {
    pub fn new(interval: u64, job_fn: F) -> Self {
        Self {
          interval,
          job_fn,
        }
    }
}

#[shuttle_runtime::async_trait]
impl<F: AsyncFunc> shuttle_runtime::Service for CronService<F> {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(self.interval)).await;
            (self.job_fn)();
        }
        Ok(())
    }
}
