use chrono::{DateTime, FixedOffset};
use sqlx::{
    postgres::{PgArguments, PgRow},
    query::{Query, QueryAs},
    Postgres,
};
use uuid::Uuid;

#[derive(Clone)]
pub enum SqlxBinds {
    String(String),
    OptionString(Option<String>),
    Int(i32),
    Bool(bool),
    Uuid(Uuid),
    DateTimeFixedOffset(DateTime<FixedOffset>),
}

pub fn binds_query(stmt: &str, binds: Vec<SqlxBinds>) -> Query<'_, Postgres, PgArguments> {
    let mut q: Query<'_, Postgres, PgArguments> = sqlx::query(stmt);
    for bind in binds.iter() {
        q = match bind {
            SqlxBinds::String(val) => q.bind(val.clone()),
            SqlxBinds::OptionString(val) => q.bind(val.clone()),
            SqlxBinds::Int(val) => q.bind(*val),
            SqlxBinds::Bool(val) => q.bind(*val),
            SqlxBinds::Uuid(val) => q.bind(*val),
            SqlxBinds::DateTimeFixedOffset(val) => q.bind(*val),
        };
    }
    q
}

pub fn binds_query_as<'a, T: for<'r> sqlx::FromRow<'r, PgRow>>(
    stmt: &'a str,
    binds: Vec<SqlxBinds>,
) -> QueryAs<'a, Postgres, T, PgArguments> {
    let mut q: QueryAs<'_, Postgres, T, PgArguments> = sqlx::query_as(stmt);
    for bind in binds.iter() {
        q = match bind {
            SqlxBinds::String(val) => q.bind(val.clone()),
            SqlxBinds::OptionString(val) => q.bind(val.clone()),
            SqlxBinds::Int(val) => q.bind(*val),
            SqlxBinds::Bool(val) => q.bind(*val),
            SqlxBinds::Uuid(val) => q.bind(*val),
            SqlxBinds::DateTimeFixedOffset(val) => q.bind(*val),
        };
    }
    q
}

pub fn query_builder(
    select: Option<String>,
    table_name: &str,
    wheres: &[String],
    order_by: Vec<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> String {
    // Select
    let mut stmt = "SELECT ".to_string();
    if let Some(val) = select {
        stmt.push_str(&val);
    } else {
        stmt.push_str(" *");
    }

    // From
    stmt.push_str(format!(" FROM {}", table_name).as_str());

    // Where
    if !wheres.is_empty() {
        stmt.push_str(" WHERE ");
        for (idx, item) in wheres.iter().enumerate() {
            stmt.push_str(&format!(" {}", item.clone()).to_string());
            if idx < wheres.len() - 1 {
                stmt.push_str(" AND");
            }
        }
    }

    // order by
    if !order_by.is_empty() {
        stmt.push_str(" ORDER BY");
        for (idx, item) in order_by.iter().enumerate() {
            stmt.push_str(format!(" {}", item).as_str());
            if idx < order_by.len() - 1 {
                stmt.push(',');
            }
        }
    }

    // Limit
    if limit.is_some() {
        stmt.push_str(format!(" LIMIT {}", limit.unwrap()).as_str());
    }

    // Offset
    if offset.is_some() {
        stmt.push_str(format!(" OFFSET {}", offset.unwrap()).as_str());
    }
    stmt
}
