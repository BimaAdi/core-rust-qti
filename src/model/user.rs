use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub user_name: String,
    pub password: String,
    pub is_2faenabled: Option<bool>,
    pub created_date: Option<DateTime<FixedOffset>>,
    pub updated_date: Option<DateTime<FixedOffset>>,
    pub deleted_date: Option<DateTime<FixedOffset>>,
}
