use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::HashMap;
use crate::errors::SearchResult;

#[derive(sqlx::FromRow, Debug)]
pub struct Outcome {
    pub id: Uuid,
    pub url: String,
    pub page_rank: f64,
}

pub async fn connect_to_db() -> SearchResult<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn avg_score(
    pool: &PgPool,
    bm25_scores: &HashMap<Uuid, f64>,
) -> SearchResult<HashMap<Uuid, (String, f64)>> {
    let doc_ids: Vec<Uuid> = bm25_scores.keys().cloned().collect();

    let outcomes = sqlx::query_as::<_, Outcome>(
        "SELECT id, url, page_rank FROM webpages WHERE id = ANY($1) AND processed = true"
    )
        .bind(&doc_ids)
        .fetch_all(pool)
        .await?;

    let mut average_scores = HashMap::new();
    for outcome in &outcomes {
        if let Some(bm25_score) = bm25_scores.get(&outcome.id) {
            let avg_score = (0.6 * *bm25_score + 0.4 * outcome.page_rank) / 2.0;
            average_scores.insert(outcome.id, (outcome.url.clone(), avg_score));
        }
    }

    Ok(average_scores)
}
