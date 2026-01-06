use async_trait::async_trait;
use sqlx::{Pool, Postgres, MySql, Sqlite, Executor, Database};
pub use sqlx;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KqlDialect {
    Postgres,
    MySql,
    Sqlite,
}

pub enum KqlPool {
    Postgres(Pool<Postgres>),
    MySql(Pool<MySql>),
    Sqlite(Pool<Sqlite>),
}

impl KqlPool {
    pub fn dialect(&self) -> KqlDialect {
        match self {
            KqlPool::Postgres(_) => KqlDialect::Postgres,
            KqlPool::MySql(_) => KqlDialect::MySql,
            KqlPool::Sqlite(_) => KqlDialect::Sqlite,
        }
    }

    pub async fn connect(dialect: KqlDialect, url: &str) -> Result<Self, sqlx::Error> {
        match dialect {
            KqlDialect::Postgres => Ok(KqlPool::Postgres(Pool::<Postgres>::connect(url).await?)),
            KqlDialect::MySql => Ok(KqlPool::MySql(Pool::<MySql>::connect(url).await?)),
            KqlDialect::Sqlite => Ok(KqlPool::Sqlite(Pool::<Sqlite>::connect(url).await?)),
        }
    }

    pub fn as_postgres(&self) -> Option<&Pool<Postgres>> {
        if let KqlPool::Postgres(p) = self { Some(p) } else { None }
    }

    pub fn as_mysql(&self) -> Option<&Pool<MySql>> {
        if let KqlPool::MySql(p) = self { Some(p) } else { None }
    }

    pub fn as_sqlite(&self) -> Option<&Pool<Sqlite>> {
        if let KqlPool::Sqlite(p) = self { Some(p) } else { None }
    }
}

pub trait KqlEntity: for<'r> sqlx::FromRow<'r, sqlx::any::AnyRow> {
    type Id: Send + Sync;
    fn table_name() -> &'static str;
}

#[async_trait]
pub trait KqlRepository<T: KqlEntity> {
    async fn find_by_id(&self, id: T::Id) -> Result<Option<T>, sqlx::Error>;
    async fn insert(&self, entity: &T) -> Result<(), sqlx::Error>;
    async fn update(&self, entity: &T) -> Result<(), sqlx::Error>;
    async fn delete(&self, id: T::Id) -> Result<(), sqlx::Error>;
    async fn list(&self) -> Result<Vec<T>, sqlx::Error>;
    
    fn query(&self) -> QueryBuilder<T>;
}

pub struct QueryBuilder<'a, T: KqlEntity> {
    pool: &'a KqlPool,
    table: &'static str,
    limit: Option<usize>,
    offset: Option<usize>,
    order_by: Vec<String>,
    conditions: Vec<String>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: KqlEntity> QueryBuilder<'a, T> {
    pub fn new(pool: &'a KqlPool) -> Self {
        Self {
            pool,
            table: T::table_name(),
            limit: None,
            offset: None,
            order_by: Vec::new(),
            conditions: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn order_by(mut self, col: &str, asc: bool) -> Self {
        let dir = if asc { "ASC" } else { "DESC" };
        self.order_by.push(format!("{} {}", col, dir));
        self
    }

    pub fn filter(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    fn build_sql(&self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table);
        
        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        sql
    }

    pub async fn all(&self) -> Result<Vec<T>, sqlx::Error> {
        let sql = self.build_sql();
        match self.pool {
            KqlPool::Postgres(p) => sqlx::query_as::<Postgres, T>(&sql).fetch_all(p).await,
            KqlPool::MySql(p) => sqlx::query_as::<MySql, T>(&sql).fetch_all(p).await,
            KqlPool::Sqlite(p) => sqlx::query_as::<Sqlite, T>(&sql).fetch_all(p).await,
        }
    }

    pub async fn first(&self) -> Result<Option<T>, sqlx::Error> {
        let mut sql = format!("SELECT * FROM {}", self.table);
        sql.push_str(" LIMIT 1");
        match self.pool {
            KqlPool::Postgres(p) => sqlx::query_as::<Postgres, T>(&sql).fetch_optional(p).await,
            KqlPool::MySql(p) => sqlx::query_as::<MySql, T>(&sql).fetch_optional(p).await,
            KqlPool::Sqlite(p) => sqlx::query_as::<Sqlite, T>(&sql).fetch_optional(p).await,
        }
    }
}
