use fake::{Dummy, Fake, Faker};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::model::permission_attribute_list::{PermissionAttributeList, TABLE_NAME};

pub struct PermissionAttributeListFactory<T: Clone> {
    modifier_one: fn(x: &PermissionAttributeList, ext: T) -> PermissionAttributeList,
    modifier_many: fn(x: &PermissionAttributeList, idx: usize, ext: T) -> PermissionAttributeList,
}

impl<T: Clone> Default for PermissionAttributeListFactory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> PermissionAttributeListFactory<T> {
    pub fn new() -> Self {
        Self {
            modifier_one: |x, _| x.clone(),
            modifier_many: |x, _, _| x.clone(),
        }
    }

    pub fn modified_one(
        &mut self,
        modifier: fn(x: &PermissionAttributeList, ext: T) -> PermissionAttributeList,
    ) {
        self.modifier_one = modifier
    }

    pub fn modified_many(
        &mut self,
        modifier: fn(x: &PermissionAttributeList, idx: usize, ext: T) -> PermissionAttributeList,
    ) {
        self.modifier_many = modifier
    }

    pub async fn generate_one(
        &mut self,
        db: &PgPool,
        ext: T,
    ) -> anyhow::Result<PermissionAttributeList> {
        let data = PermissionAttributeListDummy::new();
        let data = data.generate_one();
        let data = (self.modifier_one)(&data, ext);
        sqlx::query(
            format!(
                r#"
        INSERT INTO {} (permission_id, attribute_id) 
        VALUES ($1, $2)"#,
                TABLE_NAME
            )
            .as_str(),
        )
        .bind(data.permission_id)
        .bind(data.attribute_id)
        .execute(db)
        .await?;
        Ok(data.clone())
    }

    pub async fn generate_many(
        &mut self,
        db: &PgPool,
        num: u32,
        ext: T,
    ) -> anyhow::Result<Vec<PermissionAttributeList>> {
        let data = PermissionAttributeListDummy::new();
        let data = data.generate_many(num);
        let mut result: Vec<PermissionAttributeList> = vec![];
        for (idx, item) in data.iter().enumerate() {
            result.push((self.modifier_many)(item, idx, ext.clone()));
        }
        let mut tx = db.begin().await?;
        for item in result.clone() {
            sqlx::query(
                format!(
                    r#"
            INSERT INTO {} (permission_id, attribute_id) 
            VALUES ($1, $2)"#,
                    TABLE_NAME
                )
                .as_str(),
            )
            .bind(item.permission_id)
            .bind(item.attribute_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(result)
    }
}

#[derive(Debug, Default, Deserialize, Dummy, Clone)]
struct PermissionAttributeListDummy {}

impl PermissionAttributeListDummy {
    pub fn new() -> Self {
        Faker.fake::<Self>()
    }

    pub fn generate_one(&self) -> PermissionAttributeList {
        PermissionAttributeList {
            permission_id: Uuid::now_v7(),
            attribute_id: Uuid::now_v7(),
        }
    }

    pub fn generate_many(&self, num: u32) -> Vec<PermissionAttributeList> {
        let mut result: Vec<PermissionAttributeList> = vec![];
        for _ in 0..num {
            result.push(PermissionAttributeList {
                permission_id: Uuid::now_v7(),
                attribute_id: Uuid::now_v7(),
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
        factory::{
            permission::PermissionFactory, permission_attribute::PermissionAttributeFactory,
            permission_attribute_list::PermissionAttributeListFactory,
        },
        model::permission_attribute_list::{PermissionAttributeList, TABLE_NAME},
    };

    // Must use modified since it's foreign key related

    #[derive(Clone)]
    struct ExtData {
        pub permission_id: Uuid,
        pub attribute_id: Uuid,
    }

    #[sqlx::test]
    async fn test_generate_one_modified(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut permission_factory = PermissionFactory::new();
        let permission = permission_factory.generate_one(&pool, ()).await?;
        let mut permission_attribute_factory = PermissionAttributeFactory::new();
        let permission_attribute = permission_attribute_factory.generate_one(&pool, ()).await?;
        let mut factory = PermissionAttributeListFactory::<ExtData>::new();
        let ext = ExtData {
            permission_id: permission.id,
            attribute_id: permission_attribute.id,
        };
        factory.modified_one(|_, ext| PermissionAttributeList {
            permission_id: ext.permission_id,
            attribute_id: ext.attribute_id,
        });
        factory.generate_one(&pool, ext.clone()).await?;

        // Expect
        let res: Option<PermissionAttributeList> =
            sqlx::query_as(format!(r#"SELECT * FROM {}"#, TABLE_NAME).as_str())
                .fetch_optional(&pool)
                .await?;
        assert!(res.is_some());
        let res = res.unwrap();
        assert_eq!(res.permission_id, ext.permission_id);
        assert_eq!(res.attribute_id, ext.attribute_id);
        Ok(())
    }

    #[sqlx::test]
    async fn test_generate_many_modified(pool: PgPool) -> anyhow::Result<()> {
        // When
        let mut permission_factory = PermissionFactory::new();
        let permission = permission_factory.generate_many(&pool, 5, ()).await?;
        let mut permission_attribute_factory = PermissionAttributeFactory::new();
        let permission_attribute = permission_attribute_factory
            .generate_many(&pool, 5, ())
            .await?;
        let mut factory = PermissionAttributeListFactory::<Vec<ExtData>>::new();
        let mut ext: Vec<ExtData> = vec![];
        for i in 0..5 {
            ext.push(ExtData {
                permission_id: permission[i].id,
                attribute_id: permission_attribute[i].id,
            });
        }
        factory.modified_many(|_, idx, ext| PermissionAttributeList {
            permission_id: ext[idx].permission_id,
            attribute_id: ext[idx].attribute_id,
        });
        factory.generate_many(&pool, 5, ext.clone()).await?;

        // Expect
        let res: Vec<PermissionAttributeList> =
            sqlx::query_as(format!("SELECT * FROM {}", TABLE_NAME).as_str())
                .fetch_all(&pool)
                .await?;
        assert_eq!(res.len(), 5);
        for item in ext {
            let res: Option<PermissionAttributeList> = sqlx::query_as(
                format!(
                    "SELECT * FROM {} WHERE permission_id = $1 AND attribute_id = $2",
                    TABLE_NAME
                )
                .as_str(),
            )
            .bind(item.permission_id)
            .bind(item.attribute_id)
            .fetch_optional(&pool)
            .await?;
            assert!(res.is_some());
        }
        Ok(())
    }
}
