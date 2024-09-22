//schema.rs

use tantivy::schema::*;

pub fn create_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);

    schema_builder.add_text_field("content", TEXT);

    schema_builder.add_text_field("url", STRING | STORED);

    // Optional: Timestamp for freshness in ranking (e.g., last crawled date)
    // schema_builder.add_date_field("timestamp", FAST | STORED);

    schema_builder.build()
}
