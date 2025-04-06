use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use sqlx::prelude::FromRow;
use uuid::Uuid;

pub const TABLE_NAME: &str = "public.permission";

#[derive(Clone, Debug, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub permission_name: String,
    pub is_user: Option<bool>,
    pub is_role: Option<bool>,
    pub is_group: Option<bool>,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_date: Option<DateTime<FixedOffset>>,
    pub updated_date: Option<DateTime<FixedOffset>>,
}
