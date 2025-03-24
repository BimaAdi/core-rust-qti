use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

pub const TABLE_NAME: &str = "public.group";

#[derive(Clone, Debug, Deserialize, FromRow)]
pub struct Group {
    pub id: Uuid,
    pub group_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_date: Option<DateTime<FixedOffset>>,
    pub updated_date: Option<DateTime<FixedOffset>>,
    pub deleted_date: Option<DateTime<FixedOffset>>,
}
