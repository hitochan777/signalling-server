use anyhow::Result;
use std::{future::Future, pin::Pin};

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
