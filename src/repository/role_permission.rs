use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    core::sqlx_utils::{binds_query_as, query_builder, SqlxBinds},
    model::role_permission::{RolePermission, TABLE_NAME},
};

pub async fn get_all_role_permission(
    tx: &mut Transaction<'_, Postgres>,
    page: Option<u32>,
    page_size: Option<u32>,
    role_id: &Uuid,
    all: Option<bool>,
) -> anyhow::Result<(Vec<RolePermission>, u32, u32)> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let all = all.unwrap_or(false);
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec![];

    binds.push(SqlxBinds::Uuid(*role_id));
    filters.push(format!("role_id = ${}", binds.len()));

    let limit = match all {
        true => None,
        false => Some(page_size),
    };
    let offset = match all {
        true => None,
        false => Some((page - 1) * page_size),
    };

    let stmt = query_builder(
        None,
        TABLE_NAME,
        &filters,
        vec!["updated_date DESC".to_string()],
        limit,
        offset,
    );
    let stmt_count = query_builder(
        Some("count(*)".to_string()),
        TABLE_NAME,
        &filters,
        vec![],
        None,
        None,
    );

    let q = binds_query_as::<RolePermission>(&stmt, binds.clone());
    let q_count = binds_query_as::<(i64,)>(&stmt_count, binds);
    let data = q.fetch_all(&mut **tx).await?;
    let count = q_count.fetch_one(&mut **tx).await?;
    let num_page = match all {
        true => 0,
        false => (count.0 as u32).div_ceil(page_size),
    };
    Ok((data, count.0 as u32, num_page as u32))
}

pub async fn get_detail_role_permission(
    tx: &mut Transaction<'_, Postgres>,
    role_id: &Uuid,
    permission_id: &Uuid,
    attribute_id: &Uuid,
) -> anyhow::Result<Option<RolePermission>> {
    Ok(sqlx::query_as(
        format!(
            "SELECT * FROM {} WHERE role_id = $1 AND permission_id = $2 AND attribute_id = $3",
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(role_id)
    .bind(permission_id)
    .bind(attribute_id)
    .fetch_optional(&mut **tx)
    .await?)
}

pub async fn create_role_permission(
    tx: &mut Transaction<'_, Postgres>,
    role_permission: &RolePermission,
) -> anyhow::Result<()> {
    sqlx::query(format!("INSERT INTO {} (role_id, permission_id, attribute_id, created_by, updated_by, created_date, updated_date) VALUES ($1, $2, $3, $4, $5, $6, $7)", TABLE_NAME).as_str())
        .bind(role_permission.role_id)
        .bind(role_permission.permission_id)
        .bind(role_permission.attribute_id)
        .bind(role_permission.created_by)
        .bind(role_permission.updated_by)
        .bind(role_permission.created_date)
        .bind(role_permission.updated_date)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn delete_role_permission(
    tx: &mut Transaction<'_, Postgres>,
    role_permission: &RolePermission,
) -> anyhow::Result<()> {
    sqlx::query(
        format!(
            "DELETE FROM {} WHERE role_id = $1 AND permission_id = $2 AND attribute_id = $3",
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(role_permission.role_id)
    .bind(role_permission.permission_id)
    .bind(role_permission.attribute_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
