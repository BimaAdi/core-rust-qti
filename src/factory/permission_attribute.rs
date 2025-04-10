use chrono::{DateTime, FixedOffset};
use fake::{Dummy, Fake, Faker};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::model::permission_attribute::{PermissionAttribute, TABLE_NAME};

pub struct PermissionAttributeFactory<T: Clone> {
    modifier_one: fn(x: &PermissionAttribute, ext: T) -> PermissionAttribute,
    modifier_many: fn(x: &PermissionAttribute, idx: usize, ext: T) -> PermissionAttribute,
}

impl<T: Clone> Default for PermissionAttributeFactory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> PermissionAttributeFactory<T> {
    pub fn new() -> Self {
        Self {
            modifier_one: |x, _| x.clone(),
            modifier_many: |x, _, _| x.clone(),
        }
    }

    pub fn modified_one(
        &mut self,
        modifier: fn(x: &PermissionAttribute, ext: T) -> PermissionAttribute,
    ) {
        self.modifier_one = modifier
    }

    pub fn modified_many(
        &mut self,
        modifier: fn(x: &PermissionAttribute, idx: usize, ext: T) -> PermissionAttribute,
    ) {
        self.modifier_many = modifier
    }

    pub async fn generate_one(
        &mut self,
        db: &PgPool,
        ext: T,
    ) -> anyhow::Result<PermissionAttribute> {
        let data = PermissionAttributeDummy::new();
        let data = data.generate_one();
        let data = (self.modifier_one)(&data, ext);
        sqlx::query(
            format!(
                r#"
        INSERT INTO {} (id, name, description, created_date, updated_date) 
        VALUES ($1, $2, $3, $4, $5)"#,
                TABLE_NAME
            )
            .as_str(),
        )
        .bind(data.id)
        .bind(&data.name)
        .bind(&data.description)
        .bind(data.created_date)
        .bind(data.updated_date)
        .execute(db)
        .await?;
        Ok(data.clone())
    }

    pub async fn generate_many(
        &mut self,
        db: &PgPool,
        num: u32,
        ext: T,
    ) -> anyhow::Result<Vec<PermissionAttribute>> {
        let data = PermissionAttributeDummy::new();
        let data = data.generate_many(num);
        let mut result: Vec<PermissionAttribute> = vec![];
        for (idx, item) in data.iter().enumerate() {
            result.push((self.modifier_many)(item, idx, ext.clone()));
        }
        let mut tx = db.begin().await?;
        for item in result.clone() {
            sqlx::query(
                format!(
                    r#"
            INSERT INTO {} (id, name, description, created_date, updated_date) 
            VALUES ($1, $2, $3, $4, $5)"#,
                    TABLE_NAME
                )
                .as_str(),
            )
            .bind(item.id)
            .bind(&item.name)
            .bind(&item.description)
            .bind(item.created_date)
            .bind(item.updated_date)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(result)
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize, Dummy, Clone)]
struct PermissionAttributeDummy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_date: Option<DateTime<FixedOffset>>,
    pub updated_date: Option<DateTime<FixedOffset>>,
}

impl PermissionAttributeDummy {
    pub fn new() -> Self {
        Faker.fake::<Self>()
    }

    pub fn generate_one(&self) -> PermissionAttribute {
        let dummy = Faker.fake::<PermissionAttributeDummy>();
        PermissionAttribute {
            id: dummy.id,
            name: dummy.name,
            description: dummy.description,
            created_date: Some(Faker.fake::<DateTime<FixedOffset>>()),
            updated_date: Some(Faker.fake::<DateTime<FixedOffset>>()),
        }
    }

    pub fn generate_many(&self, num: u32) -> Vec<PermissionAttribute> {
        let mut result: Vec<PermissionAttribute> = vec![];
        for _ in 0..num {
            let dummy = Faker.fake::<Self>();
            result.push(PermissionAttribute {
                id: dummy.id,
                name: dummy.name,
                description: dummy.description,
                created_date: Some(Faker.fake::<DateTime<FixedOffset>>()),
                updated_date: Some(Faker.fake::<DateTime<FixedOffset>>()),
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

    use crate::{
        core::utils::{datetime_to_string, datetime_to_string_opt},
        factory::permission_attribute::PermissionAttributeFactory,
        model::permission_attribute::{PermissionAttribute, TABLE_NAME},
    };

    #[derive(Clone)]
    struct ExtData {
        pub id: Uuid,
        pub created_date: DateTime<FixedOffset>,
        pub updated_date: DateTime<FixedOffset>,
    }

    #[sqlx::test]
    async fn test_generate_one(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = PermissionAttributeFactory::new();
        factory.generate_one(&pool, ()).await?;

        // Expect
        let num_data: (i64,) =
            sqlx::query_as(format!(r#"SELECT COUNT(*) FROM {}"#, TABLE_NAME).as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(num_data.0, 1);
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_one_modified(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = PermissionAttributeFactory::<ExtData>::new();
        factory.modified_one(|_, ext| PermissionAttribute {
            id: ext.id,
            name: "test_permission".to_string(),
            description: Some("description".to_string()),
            created_date: Some(ext.created_date),
            updated_date: Some(ext.updated_date),
        });
        let now = Local::now().fixed_offset();
        let ext = ExtData {
            id: Uuid::now_v7(),
            created_date: now,
            updated_date: now,
        };
        factory.generate_one(&pool, ext.clone()).await?;

        // Expect
        let res: Option<PermissionAttribute> =
            sqlx::query_as(format!(r#"SELECT * FROM {}"#, TABLE_NAME).as_str())
                .fetch_optional(&pool)
                .await?;
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.name, "test_permission".to_string());
        assert_eq!(res.description, Some("description".to_string()));
        assert_eq!(
            datetime_to_string_opt(res.created_date),
            Some(datetime_to_string(ext.created_date))
        );
        assert_eq!(
            datetime_to_string_opt(res.updated_date),
            Some(datetime_to_string(ext.updated_date))
        );
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_many(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = PermissionAttributeFactory::new();
        factory.generate_many(&pool, 10, ()).await?;

        // Expect
        let num_data: (i64,) =
            sqlx::query_as(format!(r#"SELECT COUNT(*) FROM {}"#, TABLE_NAME).as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(num_data.0, 10);
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_many_modified(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = PermissionAttributeFactory::<ExtData>::new();
        factory.modified_many(|data, _, ext| PermissionAttribute {
            id: data.id,
            name: data.name.clone(),
            description: Some("description".to_string()),
            created_date: Some(ext.created_date),
            updated_date: Some(ext.updated_date),
        });
        let now = Local::now().fixed_offset();
        let ext = ExtData {
            id: Uuid::now_v7(),
            created_date: now,
            updated_date: now,
        };
        factory.generate_many(&pool, 5, ext.clone()).await?;

        // Expect
        let res: Vec<PermissionAttribute> =
            sqlx::query_as(format!(r#"SELECT * FROM {}"#, TABLE_NAME).as_str())
                .fetch_all(&pool)
                .await?;
        assert_eq!(res.len(), 5);
        for item in res {
            assert_eq!(item.description, Some("description".to_string()));
            assert_eq!(
                datetime_to_string_opt(item.created_date),
                Some(datetime_to_string(ext.created_date))
            );
            assert_eq!(
                datetime_to_string_opt(item.updated_date),
                Some(datetime_to_string(ext.updated_date))
            );
        }
        Ok(())
    }
}
