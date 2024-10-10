use actix_web::{HttpResponse, post, Responder, web};
use crate::monitors;

use crate::models::{CronJobsMonitor, JobsMonitor};

#[post("/cronjobs/monitors")]
pub async fn post_cronjobs_monitors(req_body: web::Json<CronJobsMonitor>) -> impl Responder {
    let cronjob_monitor = req_body.0;
    monitors::add_cronjobs_monitor(cronjob_monitor);
    HttpResponse::Created()
}

#[post("/jobs/monitors")]
pub async fn post_jobs_monitors(req_body: web::Json<JobsMonitor>) -> impl Responder {
    println!("adding jobs monitor {:#?}", req_body);
    let job_monitor = req_body.0;
    monitors::add_jobs_monitor(job_monitor);
    HttpResponse::Created()
}