#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("database error: {0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
