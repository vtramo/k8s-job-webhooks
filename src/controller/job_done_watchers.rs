use actix_web::{get, HttpResponse, post, Responder, web};

use crate::models::CreateJobDoneWatcherRequest;
use crate::service;

#[post("/job-done-watchers")]
async fn post_job_done_watchers(job_done_watcher: web::Json<CreateJobDoneWatcherRequest>) -> impl Responder {
    let create_job_done_watcher_request = job_done_watcher.0;
    let created_job_done_watcher = service::job_done_watchers::create_job_done_watcher(create_job_done_watcher_request).await;
    HttpResponse::Created().json(created_job_done_watcher)
}

#[get("/job-done-watchers")]
async fn get_job_done_watchers() -> impl Responder {
    let job_done_watchers = service::job_done_watchers::get_job_done_watchers().await;
    HttpResponse::Created().json(job_done_watchers)
}

#[get("/job-done-watchers/{id}")]
async fn get_job_done_watcher(id: web::Path<String>) -> impl Responder {
    match service::job_done_watchers::get_job_done_watcher_by_id(id.as_str()).await {
        None => HttpResponse::NotFound().finish(),
        Some(job_done_watcher) => HttpResponse::Ok().json(job_done_watcher),
    }
}