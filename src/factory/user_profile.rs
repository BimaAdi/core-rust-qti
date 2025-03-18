use fake::{Dummy, Fake, Faker};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::model::user_profile::UserProfile;

pub struct UserProfileFactory<T: Clone> {
    modifier_one: fn(x: &UserProfile, ext: T) -> UserProfile,
    modifier_many: fn(x: &UserProfile, idx: usize, ext: T) -> UserProfile,
}

impl<T: Clone> Default for UserProfileFactory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> UserProfileFactory<T> {
    pub fn new() -> Self {
        Self {
            modifier_one: |x, _| x.clone(),
            modifier_many: |x, _, _| x.clone(),
        }
    }

    pub fn modified_one(&mut self, modifier: fn(x: &UserProfile, ext: T) -> UserProfile) {
        self.modifier_one = modifier
    }

    pub fn modified_many(
        &mut self,
        modifier: fn(x: &UserProfile, idx: usize, ext: T) -> UserProfile,
    ) {
        self.modifier_many = modifier
    }

    pub async fn generate_one(&mut self, db: &PgPool, ext: T) -> anyhow::Result<UserProfile> {
        let data = UserProfileDummy::new();
        let data = data.generate_one();
        let data = (self.modifier_one)(&data, ext);
        sqlx::query(
            r#"
        INSERT INTO public.user_profile (id, user_id, first_name, last_name, address, email) 
        VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(data.id)
        .bind(data.user_id)
        .bind(&data.first_name)
        .bind(&data.last_name)
        .bind(&data.address)
        .bind(&data.email)
        .execute(db)
        .await?;
        Ok(data.clone())
    }

    pub async fn generate_many(
        &mut self,
        db: &PgPool,
        num: u32,
        ext: T,
    ) -> anyhow::Result<Vec<UserProfile>> {
        let data = UserProfileDummy::new();
        let data = data.generate_many(num);
        let mut result: Vec<UserProfile> = vec![];
        for (idx, item) in data.iter().enumerate() {
            result.push((self.modifier_many)(item, idx, ext.clone()));
        }
        let mut tx = db.begin().await?;
        for item in result.clone() {
            sqlx::query(
                r#"
            INSERT INTO public.user_profile (id, user_id, first_name, last_name, address, email) 
            VALUES ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(item.id)
            .bind(item.user_id)
            .bind(&item.first_name)
            .bind(item.last_name)
            .bind(item.address)
            .bind(item.email)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(result)
    }
}

#[derive(Debug, Default, Deserialize, Dummy, Clone)]
struct UserProfileDummy {
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
}

impl UserProfileDummy {
    pub fn new() -> Self {
        Faker.fake::<Self>()
    }

    pub fn generate_one(&self) -> UserProfile {
        let dummy = Faker.fake::<UserProfileDummy>();
        UserProfile {
            id: dummy.id,
            user_id: dummy.user_id,
            first_name: dummy.first_name,
            last_name: dummy.last_name,
            address: dummy.address,
            email: dummy.email,
        }
    }

    pub fn generate_many(&self, num: u32) -> Vec<UserProfile> {
        let mut result: Vec<UserProfile> = vec![];
        for _ in 0..num {
            let dummy = Faker.fake::<Self>();
            result.push(UserProfile {
                id: dummy.id,
                user_id: dummy.user_id,
                first_name: dummy.first_name,
                last_name: dummy.last_name,
                address: dummy.address,
                email: dummy.email,
            });
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        factory::{user::UserFactory, user_profile::UserProfileFactory},
        model::{user::User, user_profile::UserProfile},
    };

    #[sqlx::test]
    async fn test_generate_one(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut user_factory = UserFactory::<Uuid>::new();
        user_factory.modified_one(|data, ext| User {
            id: ext,
            user_name: data.user_name.clone(),
            password: data.password.clone(),
            is_2faenabled: data.is_2faenabled,
            created_date: data.created_date,
            updated_date: data.updated_date,
            deleted_date: None,
        });
        let user_id = Uuid::now_v7();
        user_factory.generate_one(&pool, user_id.clone()).await?;
        let mut factory = UserProfileFactory::<Uuid>::new();
        factory.modified_one(|data, ext| UserProfile {
            id: data.id,
            user_id: ext,
            first_name: data.first_name.clone(),
            last_name: data.last_name.clone(),
            address: data.address.clone(),
            email: data.email.clone(),
        });
        factory.generate_one(&pool, user_id).await?;

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
        let mut user_factory = UserFactory::<Uuid>::new();
        user_factory.modified_one(|data, ext| User {
            id: ext,
            user_name: data.user_name.clone(),
            password: data.password.clone(),
            is_2faenabled: data.is_2faenabled,
            created_date: data.created_date,
            updated_date: data.updated_date,
            deleted_date: None,
        });
        let user_id = Uuid::now_v7();
        user_factory.generate_one(&pool, user_id.clone()).await?;
        let mut factory = UserProfileFactory::<Uuid>::new();
        factory.modified_one(|data, ext| UserProfile {
            id: data.id,
            user_id: ext,
            first_name: Some("first".to_string()),
            last_name: data.last_name.clone(),
            address: data.address.clone(),
            email: data.email.clone(),
        });
        factory.generate_one(&pool, user_id.clone()).await?;

        // Expect
        let res: (Uuid, Uuid, Option<String>) =
            sqlx::query_as(r#"SELECT id, user_id, first_name FROM public.user_profile"#)
                .fetch_one(&pool)
                .await?;
        assert_eq!(res.1.to_string(), user_id.to_string());
        assert_eq!(res.2, Some("first".to_string()));
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_many(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut user_factory = UserFactory::<Uuid>::new();
        user_factory.modified_one(|data, ext| User {
            id: ext,
            user_name: data.user_name.clone(),
            password: data.password.clone(),
            is_2faenabled: data.is_2faenabled,
            created_date: data.created_date,
            updated_date: data.updated_date,
            deleted_date: None,
        });
        let user_id = Uuid::now_v7();
        user_factory.generate_one(&pool, user_id.clone()).await?;
        let mut factory = UserProfileFactory::new();
        factory.modified_many(|data, _, ext| UserProfile {
            id: data.id,
            user_id: ext,
            first_name: data.first_name.clone(),
            last_name: data.last_name.clone(),
            address: data.address.clone(),
            email: data.email.clone(),
        });
        factory.generate_many(&pool, 10, user_id).await?;

        // Expect
        let num_data: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM public.user_profile"#)
            .fetch_one(&pool)
            .await?;
        assert_eq!(num_data.0, 10);
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_many_modified(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut user_factory = UserFactory::<Uuid>::new();
        user_factory.modified_one(|data, ext| User {
            id: ext,
            user_name: data.user_name.clone(),
            password: data.password.clone(),
            is_2faenabled: data.is_2faenabled,
            created_date: data.created_date,
            updated_date: data.updated_date,
            deleted_date: None,
        });
        let user_id = Uuid::now_v7();
        user_factory.generate_one(&pool, user_id.clone()).await?;
        let mut factory = UserProfileFactory::<Uuid>::new();
        factory.modified_many(|data, _, ext| UserProfile {
            id: data.id,
            user_id: ext,
            first_name: Some("first".to_string()),
            last_name: Some("last".to_string()),
            address: data.address.clone(),
            email: data.email.clone(),
        });
        factory.generate_many(&pool, 5, user_id.clone()).await?;

        // Expect
        let res: Vec<(Uuid, Uuid, Option<String>, Option<String>)> = sqlx::query_as(
            r#"SELECT id, user_id, first_name, last_name
        FROM public.user_profile"#,
        )
        .fetch_all(&pool)
        .await?;
        assert_eq!(res.len(), 5);
        for item in res.iter() {
            assert_eq!(item.1.to_string(), user_id.clone().to_string());
            assert_eq!(item.2, Some("first".to_string()));
            assert_eq!(item.3, Some("last".to_string()));
        }
        Ok(())
    }
}
