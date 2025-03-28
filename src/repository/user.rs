use chrono::{DateTime, FixedOffset};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    core::sqlx_utils::{binds_query_as, query_builder, SqlxBinds},
    model::{
        user::{User, TABLE_NAME},
        user_group_roles::{UserGroupRoles, TABLE_NAME as USER_GROUP_ROLES_TABLE_NAME},
        user_profile::{UserProfile, TABLE_NAME as USER_PROFILE_TABLE_NAME},
    },
};

pub async fn get_all_user(
    tx: &mut Transaction<'_, Postgres>,
    page: u32,
    page_size: u32,
    search: Option<String>,
    exclude_soft_delete: Option<bool>,
) -> anyhow::Result<(Vec<User>, u32, u32)> {
    let mut binds: Vec<SqlxBinds> = vec![];
    let mut filters: Vec<String> = vec![];

    if search.is_some() {
        binds.push(SqlxBinds::String(format!("%{}%", search.unwrap())));
        filters.push(format!("user_name = ${}", binds.len()));
    }
    let exclude_soft_delete = exclude_soft_delete.unwrap_or(true);
    if exclude_soft_delete {
        filters.push("deleted_date IS NULL".to_string());
    }

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

    let q = binds_query_as::<User>(&stmt, binds.clone());
    let q_count = binds_query_as::<(i64,)>(&stmt_count, binds);
    let data = q.fetch_all(&mut **tx).await?;
    let count = q_count.fetch_one(&mut **tx).await?;
    let num_page = (count.0 as u32).div_ceil(page_size);
    Ok((data, count.0 as u32, num_page as u32))
}

pub async fn get_user_by_id(
    tx: &mut Transaction<'_, Postgres>,
    id: &Uuid,
    exclude_soft_delete: Option<bool>,
) -> anyhow::Result<(Option<User>, Option<UserProfile>)> {
    let binds: Vec<SqlxBinds> = vec![SqlxBinds::Uuid(*id)];
    let mut user_filters: Vec<String> = vec!["id = $1".to_string()];
    let user_profile_filters: Vec<String> = vec!["user_id = $1".to_string()];
    let exclude_soft_delete = exclude_soft_delete.unwrap_or(true);
    if exclude_soft_delete {
        user_filters.push("deleted_date is null".to_string());
    }
    let user_stmt = query_builder(None, TABLE_NAME, &user_filters, vec![], None, None);
    let user_profile_stmt = query_builder(
        None,
        USER_PROFILE_TABLE_NAME,
        &user_profile_filters,
        vec![],
        None,
        None,
    );
    let user_query = binds_query_as::<User>(&user_stmt, binds.clone());
    let user_profile_query = binds_query_as::<UserProfile>(&user_profile_stmt, binds);
    let user = user_query.fetch_optional(&mut **tx).await?;
    let user_profile = user_profile_query.fetch_optional(&mut **tx).await?;
    Ok((user, user_profile))
}

pub async fn get_user_by_username(
    tx: &mut Transaction<'_, Postgres>,
    username: &str,
) -> anyhow::Result<(Option<User>, Option<UserProfile>)> {
    let res_user: Option<User> = sqlx::query_as(
        r#"SELECT *
        FROM public.user
        WHERE user_name = $1
        "#,
    )
    .bind(username)
    .fetch_optional(&mut **tx)
    .await?;
    if res_user.is_none() {
        return Ok((None, None));
    }

    let res_user_profile: Option<UserProfile> = sqlx::query_as(
        r#"SELECT *
        FROM public.user_profile
        WHERE user_id = $1    
        "#,
    )
    .bind(res_user.clone().unwrap().id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok((res_user, res_user_profile))
}

pub async fn create_user(
    tx: &mut Transaction<'_, Postgres>,
    user: &User,
    user_profile: &UserProfile,
) -> anyhow::Result<()> {
    sqlx::query(
        format!(r#"
        INSERT INTO {} (id, user_name, password, is_active, is_2faenabled, created_by, updated_by, created_date, updated_date, deleted_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#, TABLE_NAME).as_str(),
    )
    .bind(user.id)
    .bind(&user.user_name)
    .bind(&user.password)
    .bind(user.is_active)
    .bind(user.is_2faenabled)
    .bind(user.created_by)
    .bind(user.updated_by)
    .bind(user.created_date)
    .bind(user.updated_date)
    .bind(user.deleted_date)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        format!(
            r#"
        INSERT INTO {} (id, user_id, first_name, last_name, address, email)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
            USER_PROFILE_TABLE_NAME
        )
        .as_str(),
    )
    .bind(user.id)
    .bind(user.id)
    .bind(&user_profile.first_name)
    .bind(&user_profile.last_name)
    .bind(&user_profile.address)
    .bind(&user_profile.email)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn update_user(
    tx: &mut Transaction<'_, Postgres>,
    user: &mut User,
    user_profile: &UserProfile,
    request_user: &User,
    now: &DateTime<FixedOffset>,
) -> anyhow::Result<()> {
    user.updated_by = Some(request_user.id);
    user.updated_date = Some(*now);
    sqlx::query(
        format!(
            r#"UPDATE {} 
            SET user_name = $1, password = $2, is_active = $3, is_2faenabled = $4, updated_by = $5, 
            updated_date = $6
            WHERE id = $7"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(&user.user_name)
    .bind(&user.password)
    .bind(user.is_active)
    .bind(user.is_2faenabled)
    .bind(request_user.id)
    .bind(now)
    .bind(user.id)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        format!(
            r#"UPDATE {}
            SET first_name = $1, last_name = $2, address = $3, email = $4
            WHERE user_id = $5"#,
            USER_PROFILE_TABLE_NAME
        )
        .as_str(),
    )
    .bind(&user_profile.first_name)
    .bind(&user_profile.last_name)
    .bind(&user_profile.address)
    .bind(&user_profile.email)
    .bind(user.id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn soft_delete_user(
    tx: &mut Transaction<'_, Postgres>,
    user: &mut User,
    request_user: &User,
    now: &DateTime<FixedOffset>,
) -> anyhow::Result<()> {
    user.updated_by = Some(request_user.id);
    user.deleted_date = Some(*now);
    sqlx::query(
        format!(
            r#"UPDATE {} SET updated_by = $1, deleted_date = $2
            WHERE id = $3"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(request_user.id)
    .bind(now)
    .bind(user.id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn get_user_group_roles_by_user(
    tx: &mut Transaction<'_, Postgres>,
    user: &User,
) -> anyhow::Result<Vec<UserGroupRoles>> {
    Ok(sqlx::query_as(
        format!(
            "SELECT * FROM {} WHERE user_id = $1",
            USER_GROUP_ROLES_TABLE_NAME
        )
        .as_str(),
    )
    .bind(user.id)
    .fetch_all(&mut **tx)
    .await?)
}

pub async fn upsert_user_group_roles(
    tx: &mut Transaction<'_, Postgres>,
    user: &User,
    user_group_roles: &Vec<UserGroupRoles>,
) -> anyhow::Result<()> {
    // delete existsing user_group_roles for user
    sqlx::query(
        format!(
            "DELETE FROM {} WHERE user_id = $1",
            USER_GROUP_ROLES_TABLE_NAME
        )
        .as_str(),
    )
    .bind(user.id)
    .execute(&mut **tx)
    .await?;

    // reinsert user_group_roles
    for item in user_group_roles {
        sqlx::query(
            format!(
                r#"INSERT INTO {} (id, user_id, role_id, group_id)
        VALUES ($1, $2, $3, $4)"#,
                USER_GROUP_ROLES_TABLE_NAME
            )
            .as_str(),
        )
        .bind(item.id)
        .bind(item.user_id)
        .bind(item.role_id)
        .bind(item.group_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}
