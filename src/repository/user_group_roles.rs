use sqlx::{Postgres, Transaction};

use crate::model::{
    group::Group,
    role::Role,
    user::User,
    user_group_roles::{UserGroupRoles, TABLE_NAME},
};

pub async fn get_detail_user_group_roles(
    tx: &mut Transaction<'_, Postgres>,
    user: &User,
    role: &Role,
    group: &Group,
) -> anyhow::Result<Option<UserGroupRoles>> {
    Ok(sqlx::query_as(
        format!(
            r#"SELECT * FROM {}
            WHERE user_id = $1 AND role_id = $2 AND group_id = $3"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(user.id)
    .bind(role.id)
    .bind(group.id)
    .fetch_optional(&mut **tx)
    .await?)
}

pub async fn add_user_group_roles(
    tx: &mut Transaction<'_, Postgres>,
    user_group_roles: &UserGroupRoles,
) -> anyhow::Result<()> {
    sqlx::query(
        format!(
            "INSERT INTO {} (id, user_id, role_id, group_id) 
            VALUES ($1, $2, $3, $4)",
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(user_group_roles.id)
    .bind(user_group_roles.user_id)
    .bind(user_group_roles.role_id)
    .bind(user_group_roles.group_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn delete_user_group_roles(
    tx: &mut Transaction<'_, Postgres>,
    user: &User,
    role: &Role,
    group: &Group,
) -> anyhow::Result<()> {
    sqlx::query(
        format!(
            "DELETE FROM {}
            WHERE user_id = $1 AND role_id = $2 AND group_id = $3",
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(user.id)
    .bind(role.id)
    .bind(group.id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
