use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    core::sqlx_utils::{binds_query_as, in_helper, query_builder, SqlxBinds},
    model::permission_attribute::{PermissionAttribute, TABLE_NAME},
};

pub async fn get_all_permission_attribute(
    tx: &mut Transaction<'_, Postgres>,
    page: Option<u32>,
    page_size: Option<u32>,
    search: Option<String>,
    limit: Option<u32>,
    all: Option<bool>,
) -> anyhow::Result<(Vec<PermissionAttribute>, u32, u32)> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let all = all.unwrap_or(false);
    let limit_param = limit;
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec![];
    if search.is_some() {
        binds.push(SqlxBinds::String(format!("%{}%", search.unwrap())));
        filters.push(format!("name ilike ${}", binds.len()));
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

    let q = binds_query_as::<PermissionAttribute>(&stmt, binds.clone());
    let q_count = binds_query_as::<(i64,)>(&stmt_count, binds);
    let data = q.fetch_all(&mut **tx).await?;
    let count = q_count.fetch_one(&mut **tx).await?;
    let num_page = match all {
        true => 0,
        false => (count.0 as u32).div_ceil(page_size),
    };
    Ok((data, count.0 as u32, num_page as u32))
}

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

pub async fn get_permission_attribute_by_ids(
    tx: &mut Transaction<'_, Postgres>,
    ids: Vec<Uuid>,
) -> anyhow::Result<Vec<PermissionAttribute>> {
    let mut ins: Vec<SqlxBinds> = vec![];
    for item in ids {
        ins.push(SqlxBinds::Uuid(item));
    }
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec![];
    in_helper(&mut binds, &mut filters, ins, "id");
    let stmt = query_builder(
        None,
        TABLE_NAME,
        &filters,
        vec!["updated_date DESC".to_string()],
        None,
        None,
    );
    let q = binds_query_as::<PermissionAttribute>(&stmt, binds.clone());
    let data = q.fetch_all(&mut **tx).await?;
    Ok(data)
}

pub async fn create_permission_attribute(
    tx: &mut Transaction<'_, Postgres>,
    permission_attribute: &PermissionAttribute,
) -> anyhow::Result<()> {
    sqlx::query(format!("INSERT INTO {} (id, name, description, created_date, updated_date) VALUES ($1, $2, $3, $4, $5)", TABLE_NAME).as_str())
        .bind(permission_attribute.id)
        .bind(&permission_attribute.name)
        .bind(&permission_attribute.description)
        .bind(permission_attribute.created_date)
        .bind(permission_attribute.updated_date)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn update_permission_attribute(
    tx: &mut Transaction<'_, Postgres>,
    permission_attribute: &PermissionAttribute,
) -> anyhow::Result<()> {
    sqlx::query(format!("UPDATE {} SET name = $1, description = $2, created_date = $3, updated_date = $4 WHERE id = $5", TABLE_NAME).as_str())
        .bind(&permission_attribute.name)
        .bind(&permission_attribute.description)
        .bind(permission_attribute.created_date)
        .bind(permission_attribute.updated_date)
        .bind(permission_attribute.id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn delete_permission_attribute(
    tx: &mut Transaction<'_, Postgres>,
    permission_attribute: &PermissionAttribute,
) -> anyhow::Result<()> {
    sqlx::query(format!("DELETE FROM {} WHERE id = $1", TABLE_NAME).as_str())
        .bind(permission_attribute.id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
