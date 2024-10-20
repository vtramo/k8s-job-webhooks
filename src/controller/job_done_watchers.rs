use actix_web::{get, HttpRequest, HttpResponse, post, Responder, web};

use crate::controller::{IDEMPOTENCY_KEY_HEADER, IdempotencyMap};
use crate::models::CreateJobDoneWatcherRequest;
use crate::service;

#[post("/job-done-watchers")]
async fn post_job_done_watchers(
    http_request: HttpRequest,
    app_state_idempotency_map: web::Data<IdempotencyMap>,
    job_done_watcher: web::Json<CreateJobDoneWatcherRequest>
) -> impl Responder {
    let idempotency_key_option = extract_idempotency_key_header_value(&http_request);
    if let Some(idempotency_key) = &idempotency_key_option {
        if let Some(resource_id) = app_state_idempotency_map.get_resource_id(idempotency_key) {
            if let Ok(Some(job_done_watcher)) = service::job_done_watchers::get_job_done_watcher_by_id(&resource_id).await {
                return HttpResponse::Ok().json(job_done_watcher);
            }
        }
    }

    let create_job_done_watcher_request = job_done_watcher.0;
    let created_job_done_watcher = match service::job_done_watchers::create_job_done_watcher(create_job_done_watcher_request).await {
        Ok(created_job_done_watcher) => created_job_done_watcher,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    if let Some(idempotency_key) = &idempotency_key_option {
        app_state_idempotency_map.insert(idempotency_key, &created_job_done_watcher.id);
    }

    HttpResponse::Created().json(created_job_done_watcher)
}

fn extract_idempotency_key_header_value(http_request: &HttpRequest) -> Option<String> {
    http_request.headers()
        .get(IDEMPOTENCY_KEY_HEADER)
        .and_then(|header_value|
            header_value.to_str()
                .ok()
                .map(|s| s.to_string()))
}


#[get("/job-done-watchers")]
async fn get_job_done_watchers() -> impl Responder {
    let job_done_watchers = match service::job_done_watchers::get_job_done_watchers().await {
        Ok(job_done_watchers) => job_done_watchers,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Created().json(job_done_watchers)
}

#[get("/job-done-watchers/{id}")]
async fn get_job_done_watcher(id: web::Path<String>) -> impl Responder {
    match service::job_done_watchers::get_job_done_watcher_by_id(id.as_str()).await {
        Ok(None) => HttpResponse::NotFound().finish(),
        Ok(Some(job_done_watcher)) => HttpResponse::Ok().json(job_done_watcher),
        _ => return HttpResponse::InternalServerError().finish(),
    }
}