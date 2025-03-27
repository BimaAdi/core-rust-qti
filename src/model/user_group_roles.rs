use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

pub const TABLE_NAME: &str = "public.user_group_roles";

#[derive(Clone, Debug, Deserialize, FromRow)]
pub struct UserGroupRoles {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
}
