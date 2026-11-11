//! The `audits` table entity.
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "audits")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_type: Option<String>,
    pub user_id: Option<String>,
    pub event: String,
    pub auditable_type: String,
    pub auditable_id: String,
    pub old_values: Option<Json>,
    pub new_values: Option<Json>,
    pub url: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub tags: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
