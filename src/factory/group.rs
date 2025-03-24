use chrono::{DateTime, FixedOffset};
use fake::{Dummy, Fake, Faker};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::model::group::{Group, TABLE_NAME};

pub struct GroupFactory<T: Clone> {
    modifier_one: fn(x: &Group, ext: T) -> Group,
    modifier_many: fn(x: &Group, idx: usize, ext: T) -> Group,
}

impl<T: Clone> Default for GroupFactory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> GroupFactory<T> {
    pub fn new() -> Self {
        Self {
            modifier_one: |x, _| x.clone(),
            modifier_many: |x, _, _| x.clone(),
        }
    }

    pub fn modified_one(&mut self, modifier: fn(x: &Group, ext: T) -> Group) {
        self.modifier_one = modifier
    }

    pub fn modified_many(&mut self, modifier: fn(x: &Group, idx: usize, ext: T) -> Group) {
        self.modifier_many = modifier
    }

    pub async fn generate_one(&mut self, db: &PgPool, ext: T) -> anyhow::Result<Group> {
        let data = GroupDummy::new();
        let data = data.generate_one();
        let data = (self.modifier_one)(&data, ext);
        sqlx::query(format!(r#"
        INSERT INTO {} (id, group_name, description, is_active, created_by, updated_by, created_date, updated_date, deleted_date) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#, TABLE_NAME).as_str())
        .bind(data.id)
        .bind(&data.group_name)
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
    ) -> anyhow::Result<Vec<Group>> {
        let data = GroupDummy::new();
        let data = data.generate_many(num);
        let mut result: Vec<Group> = vec![];
        for (idx, item) in data.iter().enumerate() {
            result.push((self.modifier_many)(item, idx, ext.clone()));
        }
        let mut tx = db.begin().await?;
        for item in result.clone() {
            sqlx::query(format!(r#"INSERT INTO {} (id, group_name, description, is_active, created_by, updated_by, created_date, updated_date, deleted_date) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#, TABLE_NAME).as_str())
            .bind(item.id)
            .bind(&item.group_name)
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
struct GroupDummy {
    pub id: Uuid,
    pub group_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_date: Option<DateTime<FixedOffset>>,
    pub updated_date: Option<DateTime<FixedOffset>>,
    pub deleted_date: Option<DateTime<FixedOffset>>,
}

impl GroupDummy {
    pub fn new() -> Self {
        Faker.fake::<Self>()
    }

    pub fn generate_one(&self) -> Group {
        let dummy = Faker.fake::<GroupDummy>();
        Group {
            id: dummy.id,
            group_name: dummy.group_name,
            description: dummy.description,
            is_active: dummy.is_active,
            created_by: None,
            updated_by: None,
            created_date: dummy.created_date,
            updated_date: dummy.updated_date,
            deleted_date: None,
        }
    }

    pub fn generate_many(&self, num: u32) -> Vec<Group> {
        let mut result: Vec<Group> = vec![];
        for _ in 0..num {
            let dummy = Faker.fake::<Self>();
            result.push(Group {
                id: dummy.id,
                group_name: dummy.group_name,
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

    use crate::{
        factory::group::GroupFactory,
        model::group::{Group, TABLE_NAME},
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
        let mut factory = GroupFactory::new();
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
        let mut factory = GroupFactory::<ExtData>::new();
        factory.modified_one(|data, ext| Group {
            id: ext.id,
            group_name: "test_group".to_string(),
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
            format!(
                r#"SELECT id, group_name, description, is_active
        FROM {}"#,
                TABLE_NAME
            )
            .as_str(),
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(res.0, ext.id);
        assert_eq!(res.1, "test_group".to_string());
        assert_eq!(res.2, Some("test description".to_string()));
        assert_eq!(res.3, Some(false));
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_many(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut factory = GroupFactory::new();
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
        let mut factory = GroupFactory::<ExtData>::new();
        factory.modified_many(|data, _, ext| Group {
            id: data.id,
            group_name: data.group_name.clone(),
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
            format!(
                r#"SELECT is_active
        FROM {}"#,
                TABLE_NAME
            )
            .as_str(),
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
