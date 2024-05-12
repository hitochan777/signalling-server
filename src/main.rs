use axum_router::create_axum_app;
use combined_service::CombinedService;
use cron_service::CronService;
use shuttle_axum::AxumService;

mod axum_router;
mod combined_service;
mod cron_service;
mod leader_selector;

#[shuttle_runtime::main]
async fn main() -> Result<CombinedService<fn() -> ()>, shuttle_runtime::Error> {
  Ok(CombinedService::new(
    AxumService::from(create_axum_app()),
    CronService::new(10, Box::pin(async || {
        println!("10 seconds passed");
    })),
  ))
}
