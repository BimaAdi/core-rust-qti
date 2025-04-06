use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use sqlx::prelude::FromRow;
use uuid::Uuid;

pub const TABLE_NAME: &str = "public.permission_attribute";

#[derive(Clone, Debug, Deserialize, FromRow)]
pub struct PermissionAttribute {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_date: Option<DateTime<FixedOffset>>,
    pub updated_date: Option<DateTime<FixedOffset>>,
}
