use anyhow::Ok;
use chrono::{DateTime, FixedOffset, Local};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    core::sqlx_utils::{binds_query_as, query_builder, SqlxBinds},
    model::{
        role::{Role, TABLE_NAME},
        user::User,
    },
};

pub async fn paginate_role(
    tx: &mut Transaction<'_, Postgres>,
    page: u32,
    page_size: u32,
    search: Option<String>,
) -> anyhow::Result<(Vec<Role>, u32, u32)> {
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec![];

    if search.is_some() {
        binds.push(SqlxBinds::String(format!("%{}%", search.unwrap())));
        filters.push(format!("role_name = ${}", binds.len()));
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

    let q = binds_query_as::<Role>(&stmt, binds.clone());
    let q_count = binds_query_as::<(i64,)>(&stmt_count, binds);
    let data = q.fetch_all(&mut **tx).await?;
    let count = q_count.fetch_one(&mut **tx).await?;
    let num_page = (count.0 as u32).div_ceil(page_size);
    Ok((data, count.0 as u32, num_page as u32))
}

pub async fn get_all_role(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<Vec<Role>> {
    let filters: Vec<String> = vec!["deleted_date IS NULL".to_string()];
    let stmt = query_builder(
        None,
        TABLE_NAME,
        &filters,
        vec!["updated_date DESC".to_string()],
        None,
        None,
    );
    let q = binds_query_as::<Role>(&stmt, vec![]);
    let data = q.fetch_all(&mut **tx).await?;
    Ok(data)
}

pub async fn get_dropdown_role(
    tx: &mut Transaction<'_, Postgres>,
    limit: Option<u32>,
    search: Option<String>,
) -> anyhow::Result<Vec<Role>> {
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec!["deleted_date IS NULL".to_string()];

    if search.is_some() {
        binds.push(SqlxBinds::String(format!("%{}%", search.unwrap())));
        filters.push(format!("role_name = ${}", binds.len()));
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
    let q = binds_query_as::<Role>(&stmt, vec![]);
    let data = q.fetch_all(&mut **tx).await?;
    Ok(data)
}

pub async fn get_role_by_id(
    tx: &mut Transaction<'_, Postgres>,
    id: &Uuid,
) -> anyhow::Result<Option<Role>> {
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
    let q = binds_query_as::<Role>(&stmt, binds);
    let data = q.fetch_optional(&mut **tx).await?;
    Ok(data)
}

pub async fn create_role(
    tx: &mut Transaction<'_, Postgres>,
    id: Option<Uuid>,
    role_name: String,
    description: Option<String>,
    is_active: Option<bool>,
    request_user: User,
    now: Option<DateTime<FixedOffset>>,
) -> anyhow::Result<Role> {
    let now = now.unwrap_or(Local::now().fixed_offset());
    let new_role = Role {
        id: id.unwrap_or(Uuid::now_v7()),
        role_name,
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
    INSERT INTO {} (id, role_name, description, is_active, created_by, 
    updated_by, created_date, updated_date, deleted_date)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(new_role.id)
    .bind(&new_role.role_name)
    .bind(&new_role.description)
    .bind(new_role.is_active)
    .bind(new_role.created_by)
    .bind(new_role.updated_by)
    .bind(new_role.created_date)
    .bind(new_role.updated_date)
    .bind(new_role.deleted_date)
    .execute(&mut **tx)
    .await?;
    Ok(new_role)
}

pub async fn update_role(
    tx: &mut Transaction<'_, Postgres>,
    role: &mut Role,
    role_name: String,
    description: Option<String>,
    is_active: Option<bool>,
    request_user: User,
    now: Option<DateTime<FixedOffset>>,
) -> anyhow::Result<()> {
    let now = now.unwrap_or(Local::now().fixed_offset());
    role.role_name = role_name;
    role.description = description;
    role.is_active = is_active;
    role.updated_by = Some(request_user.id);
    role.updated_date = Some(now);
    sqlx::query(
        format!(
            r#"
        UPDATE {} 
        SET role_name = $1, description = $2, is_active = $3, updated_by = $4, updated_date = $5
        WHERE id = $6"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(&role.role_name)
    .bind(&role.description)
    .bind(role.is_active)
    .bind(role.updated_by)
    .bind(role.updated_date)
    .bind(role.id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn soft_delete_role(
    tx: &mut Transaction<'_, Postgres>,
    role: &mut Role,
    request_user: User,
    now: Option<DateTime<FixedOffset>>,
) -> anyhow::Result<()> {
    let now = now.unwrap_or(Local::now().fixed_offset());
    role.updated_by = Some(request_user.id);
    role.updated_date = Some(now);
    role.deleted_date = Some(now);
    sqlx::query(
        format!(
            r#"UPDATE {}
    SET updated_by = $1, updated_date = $2, deleted_date = $3
    WHERE id = $4"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(role.updated_by)
    .bind(role.updated_date)
    .bind(role.deleted_date)
    .bind(role.id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
