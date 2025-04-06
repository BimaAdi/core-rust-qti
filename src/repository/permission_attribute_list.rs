use sqlx::{Postgres, Transaction};

use crate::model::permission_attribute_list::{PermissionAttributeList, TABLE_NAME};

pub async fn create_permission_attribute_list(
    tx: &mut Transaction<'_, Postgres>,
    permission_attribute_list: &PermissionAttributeList,
) -> anyhow::Result<()> {
    sqlx::query(
        format!(
            "INSERT INTO {} (permission_id, attribute_id) VALUES ($1, $2)",
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(permission_attribute_list.permission_id)
    .bind(permission_attribute_list.attribute_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
