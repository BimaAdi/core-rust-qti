use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    core::sqlx_utils::{binds_query_as, query_builder, SqlxBinds},
    model::permission_attribute_list::{PermissionAttributeList, TABLE_NAME},
};

pub async fn get_all_permission_attribute_list(
    tx: &mut Transaction<'_, Postgres>,
    permission_id: Option<&Uuid>,
    attribute_id: Option<&Uuid>,
) -> anyhow::Result<Vec<PermissionAttributeList>> {
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec![];

    if permission_id.is_some() {
        binds.push(SqlxBinds::Uuid(*permission_id.unwrap()));
        filters.push(format!("permission_id = ${}", binds.len()));
    }
    if attribute_id.is_some() {
        binds.push(SqlxBinds::Uuid(*attribute_id.unwrap()));
        filters.push(format!("attribute_id = ${}", binds.len()));
    }
    let stmt = query_builder(None, TABLE_NAME, &filters, vec![], None, None);
    let q = binds_query_as::<PermissionAttributeList>(&stmt, binds.clone());
    let data = q.fetch_all(&mut **tx).await?;
    Ok(data)
}

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
