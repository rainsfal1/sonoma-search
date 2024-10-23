use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;
use crate::document_models::html_Docs;

pub async fn fetch_unprocessed_docs(pool: &PgPool, limit: i64) -> Result<Vec<html_Docs>> {
    let records = sqlx::query_as!(html_Docs,
    r#"
    SELECT id,url,content,html_content,title
    FROM webpages
    WHERE processed = FALSE
    LIMIT $1
    "#,
    limit
    ).fetch_all(pool).await?;

    Ok(records)
}

pub async fn mark_as_processed(pool: &PgPool, doc_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE webpages
        SET processed = TRUE
        WHERE id = $1
        "#,
        doc_id
    )
        .execute(&pool)
        .await?;

    Ok(())
}
