use chrono::{DateTime, FixedOffset};
use fake::{Dummy, Fake, Faker};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::model::role::Role;

pub struct RoleFactory<T: Clone> {
    modifier_one: fn(x: &Role, ext: T) -> Role,
    modifier_many: fn(x: &Role, idx: usize, ext: T) -> Role,
}

impl<T: Clone> Default for RoleFactory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> RoleFactory<T> {
    pub fn new() -> Self {
        Self {
            modifier_one: |x, _| x.clone(),
            modifier_many: |x, _, _| x.clone(),
        }
    }

    pub fn modified_one(&mut self, modifier: fn(x: &Role, ext: T) -> Role) {
        self.modifier_one = modifier
    }

    pub fn modified_many(&mut self, modifier: fn(x: &Role, idx: usize, ext: T) -> Role) {
        self.modifier_many = modifier
    }

    pub async fn generate_one(&mut self, db: &PgPool, ext: T) -> anyhow::Result<Role> {
        let data = RoleDummy::new();
        let data = data.generate_one();
        let data = (self.modifier_one)(&data, ext);
        sqlx::query(r#"
        INSERT INTO public.role (id, role_name, description, is_active, created_by, updated_by, created_date, updated_date, deleted_date) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#)
        .bind(data.id)
        .bind(&data.role_name)
        .bind(&data.description)
        .bind(data.is_active)
        .bind(data.created_by)
        .bind(data.updated_by)
        .bind(data.created_date)
        .bind(data.updated_date)
        .bind(data.deleted_date)
        .execute(db).await?;
        Ok(data.clone())
    }

    pub async fn generate_many(
        &mut self,
        db: &PgPool,
        num: u32,
        ext: T,
    ) -> anyhow::Result<Vec<Role>> {
        let data = RoleDummy::new();
        let data = data.generate_many(num);
        let mut result: Vec<Role> = vec![];
        for (idx, item) in data.iter().enumerate() {
            result.push((self.modifier_many)(item, idx, ext.clone()));
        }
        let mut tx = db.begin().await?;
        for item in result.clone() {
            sqlx::query(r#"INSERT INTO public.role (id, role_name, description, is_active, created_by, updated_by, created_date, updated_date, deleted_date) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#)
            .bind(item.id)
            .bind(&item.role_name)
            .bind(&item.description)
            .bind(item.is_active)
            .bind(item.created_by)
            .bind(item.updated_by)
            .bind(item.created_date)
            .bind(item.updated_date)
            .bind(item.deleted_date)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(result)
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize, Dummy, Clone)]
struct RoleDummy {
    pub id: Uuid,
    pub role_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_date: Option<DateTime<FixedOffset>>,
    pub updated_date: Option<DateTime<FixedOffset>>,
    pub deleted_date: Option<DateTime<FixedOffset>>,
}

impl RoleDummy {
    pub fn new() -> Self {
        Faker.fake::<Self>()
    }

    pub fn generate_one(&self) -> Role {
        let dummy = Faker.fake::<RoleDummy>();
        Role {
            id: dummy.id,
            role_name: dummy.role_name,
            description: dummy.description,
            is_active: dummy.is_active,
            created_by: None,
            updated_by: None,
            created_date: dummy.created_date,
            updated_date: dummy.updated_date,
            deleted_date: None,
        }
    }

    pub fn generate_many(&self, num: u32) -> Vec<Role> {
        let mut result: Vec<Role> = vec![];
        for _ in 0..num {
            let dummy = Faker.fake::<Self>();
            result.push(Role {
                id: dummy.id,
                role_name: dummy.role_name,
                description: dummy.description,
                is_active: dummy.is_active,
                created_by: None,
                updated_by: None,
                created_date: dummy.created_date,
                updated_date: dummy.updated_date,
                deleted_date: None,
            });
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset, Local};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{factory::role::RoleFactory, model::role::Role};

    #[derive(Clone)]
    struct ExtData {
        pub id: Uuid,
        pub created_date: DateTime<FixedOffset>,
        pub updated_date: DateTime<FixedOffset>,
    }

    #[sqlx::test]
    async fn test_generate_one(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = RoleFactory::new();
        factory.generate_one(&pool, ()).await?;

        // Expect
        let num_data: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM public.role"#)
            .fetch_one(&pool)
            .await?;
        assert_eq!(num_data.0, 1);
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_one_modified(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = RoleFactory::<ExtData>::new();
        factory.modified_one(|data, ext| Role {
            id: ext.id,
            role_name: "test_role".to_string(),
            description: Some("test description".to_string()),
            is_active: Some(false),
            created_by: data.created_by,
            updated_by: data.updated_by,
            created_date: Some(ext.created_date),
            updated_date: Some(ext.updated_date),
            deleted_date: None,
        });
        let now = Local::now().fixed_offset();
        let ext = ExtData {
            id: Uuid::now_v7(),
            created_date: now,
            updated_date: now,
        };
        factory.generate_one(&pool, ext.clone()).await?;

        // Expect
        let res: (Uuid, String, Option<String>, Option<bool>) = sqlx::query_as(
            r#"SELECT id, role_name, description, is_active
        FROM public.role"#,
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(res.0, ext.id);
        assert_eq!(res.1, "test_role".to_string());
        assert_eq!(res.2, Some("test description".to_string()));
        assert_eq!(res.3, Some(false));
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_many(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = RoleFactory::new();
        factory.generate_many(&pool, 10, ()).await?;

        // Expect
        let num_data: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM public.role"#)
            .fetch_one(&pool)
            .await?;
        assert_eq!(num_data.0, 10);
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_many_modified(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = RoleFactory::<ExtData>::new();
        factory.modified_many(|data, _, ext| Role {
            id: data.id,
            role_name: data.role_name.clone(),
            description: data.description.clone(),
            is_active: Some(false),
            created_by: None,
            updated_by: None,
            created_date: Some(ext.created_date),
            updated_date: Some(ext.updated_date),
            deleted_date: None,
        });
        let now = Local::now().fixed_offset();
        let ext = ExtData {
            id: Uuid::now_v7(),
            created_date: now,
            updated_date: now,
        };
        factory.generate_many(&pool, 5, ext.clone()).await?;

        // Expect
        let res: Vec<(Option<bool>,)> = sqlx::query_as(
            r#"SELECT is_active
        FROM public.role"#,
        )
        .fetch_all(&pool)
        .await?;
        assert_eq!(res.len(), 5);
        for item in res {
            assert!(item.0.is_some());
            assert!(!item.0.unwrap())
        }
        Ok(())
    }
}
