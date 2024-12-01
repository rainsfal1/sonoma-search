use actix_web::{web, get, post, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::crawler::Crawler;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct CrawlRequest {
    pub query: String,
    pub max_depth: usize,
    pub max_pages: usize,
    #[serde(default = "default_priority")]
    pub priority: bool,
    #[serde(default = "default_force_crawl")]
    pub force_crawl: bool,
}

fn default_force_crawl() -> bool {
    false
}

fn default_priority() -> bool {
    true
}

#[derive(Serialize)]
pub struct CrawlResponse {
    pub job_id: String,
    pub status: String,
    pub message: String,
    pub has_results: bool,
    pub existing_results_count: usize,
    pub suggested_queries: Vec<String>,
}

#[post("/crawl")]
pub async fn crawl(
    crawler: web::Data<Arc<Mutex<Crawler>>>,
    request: web::Json<CrawlRequest>,
) -> impl Responder {
    let job_id = Uuid::new_v4().to_string();
    let request_query = request.query.clone();
    let max_depth = request.max_depth;
    let max_pages = request.max_pages;
    let priority = request.priority;
    let force_crawl = request.force_crawl;
    
    let crawler_guard = crawler.lock().await;
    
    // First check for existing results
    match crawler_guard.check_existing_results(&request_query).await {
        Ok(crawl_status) => {
            if !crawl_status.has_results && !force_crawl {
                // Return immediately with suggestions if no results and not forcing crawl
                return HttpResponse::Ok().json(CrawlResponse {
                    job_id,
                    status: "no_results".to_string(),
                    message: crawl_status.message,
                    has_results: false,
                    existing_results_count: 0,
                    suggested_queries: crawl_status.suggested_queries,
                });
            }
            
            // Either we have results or force_crawl is true
            let crawler_clone = crawler.clone();
            let job_id_clone = job_id.clone();
            let query_clone = request_query.clone();
            
            tokio::spawn(async move {
                let crawler_guard = crawler_clone.lock().await;
                if let Err(e) = crawler_guard.crawl_for_query(&query_clone, max_depth, max_pages).await {
                    log::error!("Crawl error for job {}: {}", job_id_clone, e);
                }
            });

            HttpResponse::Ok().json(CrawlResponse {
                job_id,
                status: "queued".to_string(),
                message: format!(
                    "Crawl job started for query: {} with max_depth: {}, max_pages: {}, priority: {}", 
                    request_query, max_depth, max_pages, priority
                ),
                has_results: crawl_status.has_results,
                existing_results_count: crawl_status.existing_results_count,
                suggested_queries: crawl_status.suggested_queries,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(CrawlResponse {
                job_id,
                status: "error".to_string(),
                message: format!("Error checking results: {}", e),
                has_results: false,
                existing_results_count: 0,
                suggested_queries: vec![],
            })
        }
    }
}

#[get("/job-status/{job_id}")]
pub async fn get_job_status(
    crawler: web::Data<Arc<Mutex<Crawler>>>,
    id: web::Path<String>
) -> impl Responder {
    let crawler = crawler.lock().await;
    let queue_size = crawler.get_queue_size().await;
    let pages_crawled = crawler.get_visited_count().await;
    
    let status = if queue_size == 0 && pages_crawled > 0 {
        "completed"
    } else if pages_crawled > 0 {
        "in_progress"
    } else {
        "starting"
    };

    HttpResponse::Ok().json(serde_json::json!({
        "job_id": id.into_inner(),
        "status": status,
        "pages_crawled": pages_crawled,
        "queue_size": queue_size
    }))
}
