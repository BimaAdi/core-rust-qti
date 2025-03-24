use anyhow::Ok;
use chrono::{DateTime, FixedOffset, Local};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    core::sqlx_utils::{binds_query_as, query_builder, SqlxBinds},
    model::{
        group::{Group, TABLE_NAME},
        user::User,
    },
};

pub async fn paginate_group(
    tx: &mut Transaction<'_, Postgres>,
    page: u32,
    page_size: u32,
    search: Option<String>,
) -> anyhow::Result<(Vec<Group>, u32, u32)> {
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec![];

    if search.is_some() {
        binds.push(SqlxBinds::String(format!("%{}%", search.unwrap())));
        filters.push(format!("group_name = ${}", binds.len()));
    }
    filters.push("deleted_date IS NULL".to_string());

    let limit = page_size;
    let offset = (page - 1) * page_size;
    let stmt = query_builder(
        None,
        TABLE_NAME,
        &filters,
        vec!["updated_date DESC".to_string()],
        Some(limit),
        Some(offset),
    );
    let stmt_count = query_builder(
        Some("count(id)".to_string()),
        TABLE_NAME,
        &filters,
        vec![],
        None,
        None,
    );

    let q = binds_query_as::<Group>(&stmt, binds.clone());
    let q_count = binds_query_as::<(i64,)>(&stmt_count, binds);
    let data = q.fetch_all(&mut **tx).await?;
    let count = q_count.fetch_one(&mut **tx).await?;
    let num_page = (count.0 as u32).div_ceil(page_size);
    Ok((data, count.0 as u32, num_page as u32))
}

pub async fn get_all_group(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<Vec<Group>> {
    let filters: Vec<String> = vec!["deleted_date IS NULL".to_string()];
    let stmt = query_builder(
        None,
        TABLE_NAME,
        &filters,
        vec!["updated_date DESC".to_string()],
        None,
        None,
    );
    let q = binds_query_as::<Group>(&stmt, vec![]);
    let data = q.fetch_all(&mut **tx).await?;
    Ok(data)
}

pub async fn get_dropdown_group(
    tx: &mut Transaction<'_, Postgres>,
    limit: Option<u32>,
    search: Option<String>,
) -> anyhow::Result<Vec<Group>> {
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec!["deleted_date IS NULL".to_string()];

    if search.is_some() {
        binds.push(SqlxBinds::String(format!("%{}%", search.unwrap())));
        filters.push(format!("group_name = ${}", binds.len()));
    }

    let limit = limit.unwrap_or(10);

    let stmt = query_builder(
        None,
        TABLE_NAME,
        &filters,
        vec!["updated_date DESC".to_string()],
        Some(limit),
        None,
    );
    let q = binds_query_as::<Group>(&stmt, vec![]);
    let data = q.fetch_all(&mut **tx).await?;
    Ok(data)
}

pub async fn get_group_by_id(
    tx: &mut Transaction<'_, Postgres>,
    id: &Uuid,
) -> anyhow::Result<Option<Group>> {
    let binds: Vec<SqlxBinds> = vec![SqlxBinds::Uuid(*id)];
    let filters: Vec<String> = vec!["id = $1".to_string(), "deleted_date IS NULL".to_string()];
    let stmt = query_builder(
        None,
        TABLE_NAME,
        &filters,
        vec!["updated_date DESC".to_string()],
        None,
        None,
    );
    let q = binds_query_as::<Group>(&stmt, binds);
    let data = q.fetch_optional(&mut **tx).await?;
    Ok(data)
}

pub async fn create_group(
    tx: &mut Transaction<'_, Postgres>,
    id: Option<Uuid>,
    group_name: String,
    description: Option<String>,
    is_active: Option<bool>,
    request_user: User,
    now: Option<DateTime<FixedOffset>>,
) -> anyhow::Result<Group> {
    let now = now.unwrap_or(Local::now().fixed_offset());
    let new_group = Group {
        id: id.unwrap_or(Uuid::now_v7()),
        group_name,
        description,
        is_active,
        created_by: Some(request_user.id),
        updated_by: Some(request_user.id),
        created_date: Some(now),
        updated_date: Some(now),
        deleted_date: None,
    };
    sqlx::query(
        format!(
            r#"
    INSERT INTO {} (id, group_name, description, is_active, created_by, 
    updated_by, created_date, updated_date, deleted_date)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(new_group.id)
    .bind(&new_group.group_name)
    .bind(&new_group.description)
    .bind(new_group.is_active)
    .bind(new_group.created_by)
    .bind(new_group.updated_by)
    .bind(new_group.created_date)
    .bind(new_group.updated_date)
    .bind(new_group.deleted_date)
    .execute(&mut **tx)
    .await?;
    Ok(new_group)
}

pub async fn update_group(
    tx: &mut Transaction<'_, Postgres>,
    group: &mut Group,
    group_name: String,
    description: Option<String>,
    is_active: Option<bool>,
    request_user: User,
    now: Option<DateTime<FixedOffset>>,
) -> anyhow::Result<()> {
    let now = now.unwrap_or(Local::now().fixed_offset());
    group.group_name = group_name;
    group.description = description;
    group.is_active = is_active;
    group.updated_by = Some(request_user.id);
    group.updated_date = Some(now);
    sqlx::query(
        format!(
            r#"
        UPDATE {} 
        SET group_name = $1, description = $2, is_active = $3, updated_by = $4, updated_date = $5
        WHERE id = $6"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(&group.group_name)
    .bind(&group.description)
    .bind(group.is_active)
    .bind(group.updated_by)
    .bind(group.updated_date)
    .bind(group.id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn soft_delete_group(
    tx: &mut Transaction<'_, Postgres>,
    group: &mut Group,
    request_user: User,
    now: Option<DateTime<FixedOffset>>,
) -> anyhow::Result<()> {
    let now = now.unwrap_or(Local::now().fixed_offset());
    group.updated_by = Some(request_user.id);
    group.updated_date = Some(now);
    group.deleted_date = Some(now);
    sqlx::query(
        format!(
            r#"UPDATE {}
    SET updated_by = $1, updated_date = $2, deleted_date = $3
    WHERE id = $4"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(group.updated_by)
    .bind(group.updated_date)
    .bind(group.deleted_date)
    .bind(group.id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
