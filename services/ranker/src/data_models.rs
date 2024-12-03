use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct LinkStorage {
    pub source_webpage_id: Uuid,
    pub target_url: String,
}

#[derive(Debug)]
pub struct UuidConversionError(pub String);

impl std::fmt::Display for UuidConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UUID Conversion Error: {}", self.0)
    }
}

impl std::error::Error for UuidConversionError {}
