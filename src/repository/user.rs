use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::model::{user::User, user_profile::UserProfile};

pub async fn get_user_by_id(
    tx: &mut Transaction<'_, Postgres>,
    id: &Uuid,
) -> anyhow::Result<(Option<User>, Option<UserProfile>)> {
    let res_user: Option<User> = sqlx::query_as(
        r#"SELECT id, user_name, password, is_2faenabled, created_date, updated_date, deleted_date
        FROM public.user
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;

    let res_user_profile: Option<UserProfile> = sqlx::query_as(
        r#"SELECT id, user_id, first_name, last_name, address, email
        FROM public.user_profile
        WHERE user_id = $1    
        "#,
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok((res_user, res_user_profile))
}

pub async fn get_user_by_username(
    tx: &mut Transaction<'_, Postgres>,
    username: &str,
) -> anyhow::Result<(Option<User>, Option<UserProfile>)> {
    let res_user: Option<User> = sqlx::query_as(
        r#"SELECT id, user_name, password, is_2faenabled, created_date, updated_date, deleted_date
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
        r#"SELECT id, user_id, first_name, last_name, address, email
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
        r#"
        INSERT INTO public.user (id, user_name, password, is_2faenabled, created_date, updated_date, deleted_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(user.id)
    .bind(&user.user_name)
    .bind(&user.password)
    .bind(user.is_2faenabled)
    .bind(user.created_date)
    .bind(user.updated_date)
    .bind(user.deleted_date)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO public.user_profile (id, user_id, first_name, last_name, address, email)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
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
