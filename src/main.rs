use axum_router::create_axum_app;
use combined_service::CombinedService;
use cron_service::CronService;
use shuttle_axum::AxumService;
use std::future::Future;

mod axum_router;
mod combined_service;
mod cron_service;
mod leader_selector;

#[shuttle_runtime::main]
async fn main() -> Result<CombinedService, shuttle_runtime::Error> {
    let job: Box<dyn Send + 'static + Fn() -> Box<dyn Send + 'static + Future<Output = ()>>> =
        Box::new(|| {
            Box::new(async {
                println!("hello");
            })
        });
    Ok(CombinedService::new(
        AxumService::from(create_axum_app()),
        CronService::new(10, job),
    ))
}
