use chrono::Local;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::security::hash_password,
    model::{user::User, user_profile::UserProfile},
    repository,
};

pub async fn create_user(pool: &PgPool, username: &str, password: &str) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    let hashed_password = hash_password(password).unwrap();
    let now = Local::now().fixed_offset();
    let user = User {
        id: Uuid::now_v7(),
        user_name: username.to_string(),
        password: hashed_password,
        is_active: Some(true),
        is_2faenabled: Some(false),
        created_by: None,
        updated_by: None,
        created_date: Some(now),
        updated_date: Some(now),
        deleted_date: None,
    };
    let user_profile = UserProfile {
        id: user.id,
        user_id: user.id,
        first_name: None,
        last_name: None,
        email: None,
        address: None,
    };
    repository::user::create_user(&mut tx, &user, &user_profile)
        .await
        .unwrap();
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::cli::auth::create_user;

    #[sqlx::test]
    async fn test_create_user(pool: PgPool) -> sqlx::Result<()> {
        // When
        let username = "test";
        let password = "test";
        create_user(&pool, username, password).await.unwrap();

        // Expect
        let db_res: Option<(String, String)> = sqlx::query_as(
            r#"
            SELECT user_name, password
            FROM public.user
            WHERE user_name = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(db_res.is_some());
        assert_eq!(db_res.unwrap().0, username);
        Ok(())
    }
}
