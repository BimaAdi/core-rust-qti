use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    core::sqlx_utils::{binds_query_as, query_builder, SqlxBinds},
    model::group_permission::{GroupPermission, TABLE_NAME},
};

pub async fn get_all_group_permission(
    tx: &mut Transaction<'_, Postgres>,
    page: Option<u32>,
    page_size: Option<u32>,
    group_id: &Uuid,
    all: Option<bool>,
) -> anyhow::Result<(Vec<GroupPermission>, u32, u32)> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let all = all.unwrap_or(false);
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec![];

    binds.push(SqlxBinds::Uuid(*group_id));
    filters.push(format!("group_id = ${}", binds.len()));

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

    let q = binds_query_as::<GroupPermission>(&stmt, binds.clone());
    let q_count = binds_query_as::<(i64,)>(&stmt_count, binds);
    let data = q.fetch_all(&mut **tx).await?;
    let count = q_count.fetch_one(&mut **tx).await?;
    let num_page = match all {
        true => 0,
        false => (count.0 as u32).div_ceil(page_size),
    };
    Ok((data, count.0 as u32, num_page as u32))
}

pub async fn get_detail_group_permission(
    tx: &mut Transaction<'_, Postgres>,
    group_id: &Uuid,
    permission_id: &Uuid,
    attribute_id: &Uuid,
) -> anyhow::Result<Option<GroupPermission>> {
    Ok(sqlx::query_as(
        format!(
            "SELECT * FROM {} WHERE group_id = $1 AND permission_id = $2 AND attribute_id = $3",
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(group_id)
    .bind(permission_id)
    .bind(attribute_id)
    .fetch_optional(&mut **tx)
    .await?)
}

pub async fn create_group_permission(
    tx: &mut Transaction<'_, Postgres>,
    group_permission: &GroupPermission,
) -> anyhow::Result<()> {
    sqlx::query(format!("INSERT INTO {} (group_id, permission_id, attribute_id, created_by, updated_by, created_date, updated_date) VALUES ($1, $2, $3, $4, $5, $6, $7)", TABLE_NAME).as_str())
        .bind(group_permission.group_id)
        .bind(group_permission.permission_id)
        .bind(group_permission.attribute_id)
        .bind(group_permission.created_by)
        .bind(group_permission.updated_by)
        .bind(group_permission.created_date)
        .bind(group_permission.updated_date)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn delete_group_permission(
    tx: &mut Transaction<'_, Postgres>,
    group_permission: &GroupPermission,
) -> anyhow::Result<()> {
    sqlx::query(
        format!(
            "DELETE FROM {} WHERE group_id = $1 AND permission_id = $2 AND attribute_id = $3",
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(group_permission.group_id)
    .bind(group_permission.permission_id)
    .bind(group_permission.attribute_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
