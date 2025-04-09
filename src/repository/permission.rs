use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    core::sqlx_utils::{binds_query_as, query_builder, SqlxBinds},
    model::permission::{Permission, TABLE_NAME},
};

#[allow(clippy::too_many_arguments)]
pub async fn get_all_permission(
    tx: &mut Transaction<'_, Postgres>,
    page: Option<u32>,
    page_size: Option<u32>,
    search: Option<String>,
    is_user: Option<bool>,
    is_role: Option<bool>,
    is_group: Option<bool>,
    limit: Option<u32>,
    all: Option<bool>,
) -> anyhow::Result<(Vec<Permission>, u32, u32)> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let all = all.unwrap_or(false);
    let limit_param = limit;
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec![];

    if search.is_some() {
        binds.push(SqlxBinds::String(format!("%{}%", search.unwrap())));
        filters.push(format!("permission_name = ${}", binds.len()));
    }
    if is_user.is_some() {
        binds.push(SqlxBinds::Bool(is_user.unwrap()));
        filters.push(format!("is_user = ${}", binds.len()));
    }
    if is_role.is_some() {
        binds.push(SqlxBinds::Bool(is_role.unwrap()));
        filters.push(format!("is_role = ${}", binds.len()));
    }
    if is_group.is_some() {
        binds.push(SqlxBinds::Bool(is_group.unwrap()));
        filters.push(format!("is_group = ${}", binds.len()));
    }

    let mut limit = match all {
        true => None,
        false => Some(page_size),
    };
    let offset = match all {
        true => None,
        false => Some((page - 1) * page_size),
    };
    if limit_param.is_some() {
        limit = limit_param;
    }
    let stmt = query_builder(
        None,
        TABLE_NAME,
        &filters,
        vec!["updated_date DESC".to_string()],
        limit,
        offset,
    );
    let stmt_count = query_builder(
        Some("count(id)".to_string()),
        TABLE_NAME,
        &filters,
        vec![],
        None,
        None,
    );

    let q = binds_query_as::<Permission>(&stmt, binds.clone());
    let q_count = binds_query_as::<(i64,)>(&stmt_count, binds);
    let data = q.fetch_all(&mut **tx).await?;
    let count = q_count.fetch_one(&mut **tx).await?;
    let num_page = match all {
        true => 0,
        false => (count.0 as u32).div_ceil(page_size),
    };
    Ok((data, count.0 as u32, num_page as u32))
}

pub async fn get_permission_by_id(
    tx: &mut Transaction<'_, Postgres>,
    id: &Uuid,
) -> anyhow::Result<Option<Permission>> {
    Ok(
        sqlx::query_as(format!("SELECT * FROM {} WHERE id = $1", TABLE_NAME).as_str())
            .bind(id)
            .fetch_optional(&mut **tx)
            .await?,
    )
}

pub async fn create_permission(
    tx: &mut Transaction<'_, Postgres>,
    permission: &Permission,
) -> anyhow::Result<()> {
    sqlx::query(
        format!(
            "INSERT INTO {} (id, permission_name, is_user, is_role, is_group, 
        description, created_by, updated_by, created_date, updated_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(permission.id)
    .bind(&permission.permission_name)
    .bind(permission.is_user)
    .bind(permission.is_role)
    .bind(permission.is_group)
    .bind(&permission.description)
    .bind(permission.created_by)
    .bind(permission.updated_by)
    .bind(permission.created_date)
    .bind(permission.updated_date)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn update_permission(
    tx: &mut Transaction<'_, Postgres>,
    permission: &Permission,
) -> anyhow::Result<()> {
    sqlx::query(
        format!(
            "UPDATE {} 
        SET permission_name = $1, is_user = $2, is_role = $3, is_group = $4, description = $5,
        created_by = $6, updated_by = $7, created_date = $8, updated_date = $9
        WHERE id = $10",
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(&permission.permission_name)
    .bind(permission.is_user)
    .bind(permission.is_role)
    .bind(permission.is_group)
    .bind(&permission.description)
    .bind(permission.created_by)
    .bind(permission.updated_by)
    .bind(permission.created_date)
    .bind(permission.updated_date)
    .bind(permission.id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn delete_permission(
    tx: &mut Transaction<'_, Postgres>,
    permission: &Permission,
) -> anyhow::Result<()> {
    sqlx::query(format!("DELETE FROM {} WHERE id = $1", TABLE_NAME).as_str())
        .bind(permission.id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
