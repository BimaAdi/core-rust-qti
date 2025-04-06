use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::model::permission_attribute::{PermissionAttribute, TABLE_NAME};

pub async fn get_permission_attribute_by_id(
    tx: &mut Transaction<'_, Postgres>,
    id: &Uuid,
) -> anyhow::Result<Option<PermissionAttribute>> {
    Ok(
        sqlx::query_as(format!("SELECT * FROM {} WHERE id = $1", TABLE_NAME).as_str())
            .bind(id)
            .fetch_optional(&mut **tx)
            .await?,
    )
}
