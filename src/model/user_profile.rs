use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
}
