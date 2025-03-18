use chrono::{DateTime, FixedOffset};
use fake::{Dummy, Fake, Faker};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::model::user::User;

pub struct UserFactory<T: Clone> {
    modifier_one: fn(x: &User, ext: T) -> User,
    modifier_many: fn(x: &User, idx: usize, ext: T) -> User,
}

impl<T: Clone> Default for UserFactory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> UserFactory<T> {
    pub fn new() -> Self {
        Self {
            modifier_one: |x, _| x.clone(),
            modifier_many: |x, _, _| x.clone(),
        }
    }

    pub fn modified_one(&mut self, modifier: fn(x: &User, ext: T) -> User) {
        self.modifier_one = modifier
    }

    pub fn modified_many(&mut self, modifier: fn(x: &User, idx: usize, ext: T) -> User) {
        self.modifier_many = modifier
    }

    pub async fn generate_one(&mut self, db: &PgPool, ext: T) -> anyhow::Result<User> {
        let data = UserDummy::new();
        let data = data.generate_one();
        let data = (self.modifier_one)(&data, ext);
        sqlx::query(r#"
        INSERT INTO public.user (id, user_name, password, is_2faenabled, created_date, updated_date, deleted_date) 
        VALUES ($1, $2, $3, $4, $5, $6, $7)"#)
        .bind(data.id)
        .bind(&data.user_name)
        .bind(&data.password)
        .bind(data.is_2faenabled)
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
    ) -> anyhow::Result<Vec<User>> {
        let data = UserDummy::new();
        let data = data.generate_many(num);
        let mut result: Vec<User> = vec![];
        for (idx, item) in data.iter().enumerate() {
            result.push((self.modifier_many)(item, idx, ext.clone()));
        }
        let mut tx = db.begin().await?;
        for item in result.clone() {
            sqlx::query(r#"INSERT INTO public.user (id, user_name, password, is_2faenabled, created_date, updated_date, deleted_date) 
            VALUES ($1, $2, $3, $4, $5, $6, $7)"#)
            .bind(item.id)
            .bind(&item.user_name)
            .bind(&item.password)
            .bind(item.is_2faenabled)
            .bind(item.created_date)
            .bind(item.updated_date)
            .bind(item.deleted_date)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(result)
    }
}

#[derive(Debug, Default, Deserialize, Dummy, Clone)]
struct UserDummy {
    pub id: Uuid,
    pub user_name: String,
    pub password: String,
    pub is_2faenabled: Option<bool>,
    pub created_date: Option<DateTime<FixedOffset>>,
    pub updated_date: Option<DateTime<FixedOffset>>,
    pub deleted_date: Option<DateTime<FixedOffset>>,
}

impl UserDummy {
    pub fn new() -> Self {
        Faker.fake::<Self>()
    }

    pub fn generate_one(&self) -> User {
        let dummy = Faker.fake::<UserDummy>();
        User {
            id: dummy.id,
            user_name: dummy.user_name,
            password: dummy.password,
            is_2faenabled: dummy.is_2faenabled,
            created_date: dummy.created_date,
            updated_date: dummy.updated_date,
            deleted_date: dummy.deleted_date,
        }
    }

    pub fn generate_many(&self, num: u32) -> Vec<User> {
        let mut result: Vec<User> = vec![];
        for _ in 0..num {
            let dummy = Faker.fake::<Self>();
            result.push(User {
                id: dummy.id,
                user_name: dummy.user_name,
                password: dummy.password,
                is_2faenabled: dummy.is_2faenabled,
                created_date: dummy.created_date,
                updated_date: dummy.updated_date,
                deleted_date: dummy.deleted_date,
            });
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset, Local};
    use fake::{Fake, Faker};
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{factory::user::UserFactory, model::user::User};

    #[derive(Clone)]
    struct ExtData {
        pub id: Uuid,
        pub created_date: DateTime<FixedOffset>,
        pub updated_date: DateTime<FixedOffset>,
    }

    #[sqlx::test]
    async fn test_generate_one(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = UserFactory::new();
        factory.generate_one(&pool, ()).await?;

        // Expect
        let num_data: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM public.user"#)
            .fetch_one(&pool)
            .await?;
        assert_eq!(num_data.0, 1);
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_one_modified(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = UserFactory::<ExtData>::new();
        factory.modified_one(|data, ext| User {
            id: ext.id,
            user_name: "test_user".to_string(),
            password: data.password.clone(),
            is_2faenabled: Some(false),
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
        let res: (
            Uuid,
            String,
            String,
            Option<bool>,
            Option<DateTime<FixedOffset>>,
            Option<DateTime<FixedOffset>>,
            Option<DateTime<FixedOffset>>,
        ) = sqlx::query_as(
            r#"SELECT id, user_name, password, is_2faenabled, 
        created_date, updated_date, deleted_date 
        FROM public.user"#,
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(res.0, ext.id);
        assert_eq!(res.1, "test_user".to_string());
        assert_ne!(res.2, "".to_string());
        assert!(!res.3.unwrap());
        assert!(res.4.is_some());
        assert!(res.5.is_some());
        assert!(res.6.is_none());
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_many(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = UserFactory::new();
        factory.generate_many(&pool, 10, ()).await?;

        // Expect
        let num_data: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM public.user"#)
            .fetch_one(&pool)
            .await?;
        assert_eq!(num_data.0, 10);
        Ok(())
    }

    fn is_deleted(is_delete: bool) -> Option<DateTime<FixedOffset>> {
        if is_delete {
            Some(Faker.fake())
        } else {
            None
        }
    }

    #[sqlx::test]
    async fn test_generate_many_modified(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = UserFactory::<ExtData>::new();
        factory.modified_many(|data, idx, ext| User {
            id: data.id,
            user_name: data.user_name.clone(),
            password: data.password.clone(),
            is_2faenabled: Some(false),
            created_date: Some(ext.created_date),
            updated_date: Some(ext.updated_date),
            deleted_date: is_deleted(idx % 2 == 0),
        });
        let now = Local::now().fixed_offset();
        let ext = ExtData {
            id: Uuid::now_v7(),
            created_date: now,
            updated_date: now,
        };
        factory.generate_many(&pool, 5, ext.clone()).await?;

        // Expect
        let res: Vec<(
            Uuid,
            String,
            String,
            Option<bool>,
            Option<DateTime<FixedOffset>>,
            Option<DateTime<FixedOffset>>,
            Option<DateTime<FixedOffset>>,
        )> = sqlx::query_as(
            r#"SELECT id, user_name, password, is_2faenabled, 
        created_date, updated_date, deleted_date
        FROM public.user"#,
        )
        .fetch_all(&pool)
        .await?;
        assert_eq!(res.len(), 5);
        for (idx, item) in res.iter().enumerate() {
            assert!(item.4.is_some());
            assert!(item.5.is_some());
            if idx % 2 == 0 {
                assert!(item.6.is_some());
            } else {
                assert!(item.6.is_none())
            }
        }
        Ok(())
    }
}
