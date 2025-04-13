use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use sqlx::prelude::FromRow;
use uuid::Uuid;

pub const TABLE_NAME: &str = "public.role_permissions";

#[derive(Clone, Debug, Deserialize, FromRow)]
pub struct RolePermission {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub attribute_id: Uuid,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_date: Option<DateTime<FixedOffset>>,
    pub updated_date: Option<DateTime<FixedOffset>>,
}
