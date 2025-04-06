use serde::Deserialize;
use sqlx::prelude::FromRow;
use uuid::Uuid;

pub const TABLE_NAME: &str = "public.permission_attribute_list";

#[derive(Clone, Debug, Deserialize, FromRow)]
pub struct PermissionAttributeList {
    pub permission_id: Uuid,
    pub attribute_id: Uuid,
}
